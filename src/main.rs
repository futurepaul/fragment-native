use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

use argh::FromArgs;
use chrono::prelude::*;
use crossbeam_channel::unbounded;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use open;

use druid::widget::{Flex, Label, List, Painter, Scroll, Split, TextBox, WidgetExt};
use druid::{
    theme, AppLauncher, Color, Data, Env, ExtEventSink, Lens, LocalizedString, RenderContext,
    Widget, WindowDesc,
};

mod search;
use search::ListItem;

mod keyup;
use keyup::KeyUp;

mod search_controller;
use search_controller::{SearchController, REFRESH_SEARCH, SEARCH_RESULTS};

#[derive(Debug)]
pub enum FragmentError {
    Io(std::io::Error),
    Druid(druid::PlatformError),
}

#[derive(FromArgs)]
/// Search notes.
struct FragmentArgs {
    /// path of the notes folder to search
    #[argh(option, short = 'p')]
    path: String,
}

#[derive(Clone, Data, Lens)]
struct FragmentState {
    results: Arc<Vec<ListItem>>,
    query: String,
    path: String,
    search_sequence: Arc<AtomicU64>,
    event_sink: Arc<ExtEventSink>,
}

impl FragmentState {
    fn search(&self) {
        let query = self.query.clone();
        let path = self.path.clone();
        let event_sink = self.event_sink.clone();
        let current_sequence = self.search_sequence.clone().fetch_add(1, Ordering::Relaxed) + 1;
        let sequence = self.search_sequence.clone();
        let (s, r) = unbounded();

        thread::spawn(move || {
            // if this fails we're shutting down
            let results =
                search::search(&query, &path, &sequence, current_sequence).expect("Search failed");
            s.send(results).unwrap();
        });
        let received_results = r.iter().last().unwrap();
        if let Err(_) = event_sink.submit_command(SEARCH_RESULTS, received_results, None) {}
    }

    fn create_note_and_open(&self) -> Result<(), FragmentError> {
        let file_with_path = std::path::Path::new(&self.path)
            .join(self.query.clone())
            .with_extension("md");
        std::fs::File::create(&file_with_path).map_err(FragmentError::Io)?;
        open::that(file_with_path).map_err(FragmentError::Io)?;
        Ok(())
    }

    fn watch(&self) -> RecommendedWatcher {
        let event_sink = self.event_sink.clone();

        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| match res {
            Ok(event) => {
                println!("event: {:?}", event);
                if let Err(_) = event_sink.submit_command(REFRESH_SEARCH, "None", None) {}
            }
            Err(e) => println!("watch error: {:?}", e),
        })
        .expect("Couldn't create watcher");

        let path = self.path.clone();

        watcher
            .watch(
                dbg!(std::path::Path::new(&path.clone())),
                RecursiveMode::Recursive,
            )
            .expect("Watcher couldn't watch");

        watcher
    }
}

fn main() -> Result<(), FragmentError> {
    // Get path argument from command line
    let args: FragmentArgs = argh::from_env();

    let main_window =
        WindowDesc::new(ui_builder).title(LocalizedString::new("").with_placeholder("Fragment"));

    let launcher = AppLauncher::with_window(main_window);

    let event_sink = launcher.get_external_handle();

    let search_sequence = AtomicU64::new(0);

    let data = FragmentState {
        results: Arc::new(
            search::search("", &args.path, &search_sequence, 1).expect("Couldn't search"),
        ),
        query: String::new(),
        path: args.path,
        search_sequence: Arc::new(search_sequence),
        event_sink: Arc::new(event_sink),
    };

    // Fire up a thread to notify of changes at the root path
    let _watch = data.watch();

    launcher.launch(data).map_err(FragmentError::Druid)?;

    Ok(())
}

fn ui_builder() -> impl Widget<FragmentState> {
    Flex::column()
        // Search box. Automatically gains focus on launch
        .with_child(build_search_box())
        // The rest of the app
        .with_flex_child(
            Split::horizontal(
                // Search results
                build_top_pane(),
                // File preview (TODO)
                build_text_pane(),
            )
            .split_point(0.8)
            .draggable(true),
            1.0,
        )
        .controller(SearchController {
            phantom: std::marker::PhantomData::default(),
        })

    // .debug_paint_layout()
}

fn build_list_item() -> impl Widget<ListItem> {
    let painter: Painter<ListItem> = Painter::new(|ctx, _, env| {
        let bounds = ctx.size().to_rect();

        if ctx.is_hot() {
            ctx.fill(bounds, &env.get(theme::PRIMARY_DARK));
        }

        if ctx.is_active() {
            ctx.fill(bounds, &env.get(theme::PRIMARY_LIGHT));
        }

        let (x, y) = (ctx.size().width, ctx.size().height);

        let mut path = druid::kurbo::BezPath::new();
        path.move_to((0.0, y));
        path.line_to((x, y));
        // Create a color
        let stroke_color = env.get(theme::BACKGROUND_LIGHT);
        // Stroke the path with thickness 1.0
        ctx.stroke(path, &stroke_color, 1.0);
    });

    Flex::column()
        .with_child(
            Flex::row()
                .with_flex_child(
                    Label::new(|data: &ListItem, _: &Env| (*data).file_name.to_string())
                        .padding(5.0)
                        .expand_width(),
                    2.0,
                )
                .with_flex_child(
                    Label::new(|data: &ListItem, _: &Env| {
                        let today = Utc::now().timestamp();
                        let seconds_old = data
                            .modified
                            .elapsed()
                            .expect("Couldn't get elapsed.")
                            .as_secs() as i64;

                        let date_modified = today - seconds_old;
                        let dt: DateTime<Utc> = Utc.timestamp(date_modified, 0);

                        dt.format("%b %e, %Y").to_string()
                    })
                    .padding(5.0)
                    .expand_width(),
                    1.0,
                ),
        )
        .with_child(
            Label::new(
                |data: &ListItem, _: &Env| match (*data).found_line.clone() {
                    Some(line) => line.to_string(),
                    None => "Couldn't read file".to_string(),
                },
            )
            .with_text_color(Color::rgb8(200, 200, 200))
            .padding(druid::Insets::new(5.0, 0.0, 0.0, 5.0))
            .expand_width(),
        )
        .background(painter)
        .on_click(|_, data, _| {
            data.open_note_in_editor();
        })
}

fn build_top_pane() -> impl Widget<FragmentState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_flex_child(
                    Label::new("Title")
                        .padding(5.0)
                        .background(Color::BLACK)
                        .expand_width(),
                    2.0,
                )
                .with_flex_child(
                    Label::new("Date Modified")
                        .padding(5.0)
                        .background(theme::PRIMARY_DARK)
                        .expand_width(),
                    1.0,
                )
                .must_fill_main_axis(true),
        )
        .with_flex_child(
            Scroll::new(List::new(|| build_list_item()).lens(FragmentState::results))
                .vertical()
                .expand_width(),
            1.0,
        )
        .border(Color::rgb8(100, 100, 100), 1.0)
        .rounded(5.0)
        .padding(5.0)
}

fn build_search_box() -> impl Widget<FragmentState> {
    TextBox::new()
        .lens(FragmentState::query)
        .controller(KeyUp::new(|_, data: &mut FragmentState, _, key_event| {
            if key_event.key_code == druid::KeyCode::Return {
                data.query = data.query.trim().to_string();
                data.create_note_and_open();
            } else {
                data.search();
            }
        }))
        .expand_width()
        .padding(5.0)
}

fn build_text_pane() -> impl Widget<FragmentState> {
    Scroll::new(
        Flex::column()
            .with_child(Label::new("Some day"))
            .with_child(Label::new("we'll have"))
            .with_child(Label::new("multiline text"))
            .with_child(Label::new("and it's going to be so great"))
            .with_child(Label::new("you just wait")),
    )
    .vertical()
    .expand_width()
}
