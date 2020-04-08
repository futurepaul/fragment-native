use argh::FromArgs;
use druid::widget::{Flex, Label, List, Painter, Scroll, Split, TextBox, WidgetExt};
use druid::{
    theme, AppLauncher, Color, Data, Env, ExtEventSink, Lens, LocalizedString, RenderContext,
    Widget, WindowDesc,
};
use std::sync::Arc;
use std::thread;

use notify::{RecommendedWatcher, RecursiveMode, Result, Watcher};

use chrono::prelude::*;

use open;

mod search;
use search::ListItem;

mod keyup;
use keyup::KeyUp;

mod search_controller;
use search_controller::{Search, REFRESH_SEARCH, SEARCH_RESULTS};

// const COUNT_BG: Color = Color::grey8(0x77);

#[derive(FromArgs)]
/// Search notes.
struct Fragment {
    /// path of the notes folder to search
    #[argh(option, short = 'p')]
    path: String,
}

#[derive(Clone, Data, Lens)]
struct FragmentState {
    results: Arc<Vec<ListItem>>,
    query: String,
    path: String,
    event_sink: Arc<ExtEventSink>,
}

impl FragmentState {
    fn search(&self) {
        let query = self.query.clone();
        let path = self.path.clone();
        let event_sink = self.event_sink.clone();
        thread::spawn(move || {
            // if this fails we're shutting down
            let results = search::search(&query, &path).unwrap();
            if let Err(_) = event_sink.submit_command(SEARCH_RESULTS, results, None) {}
        });
    }

    fn watch(&self) -> RecommendedWatcher {
        let event_sink = self.event_sink.clone();

        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| match res {
            Ok(event) => {
                println!("event: {:?}", event);
                // let results = search::search(&query, &path).unwrap();
                if let Err(_) = event_sink.submit_command(REFRESH_SEARCH, "None", None) {}
            }
            Err(e) => println!("watch error: {:?}", e),
        })
        .unwrap();

        let path = self.path.clone();

        watcher
            .watch(
                dbg!(std::path::Path::new(&path.clone())),
                RecursiveMode::Recursive,
            )
            .unwrap();

        watcher
    }
}

fn open_note_in_editor(path: &str) {
    open::that(path).unwrap();
}

fn create_note_and_open(path: &str, name: &str) {
    let file_with_path = std::path::Path::new(path).join(name).with_extension("md");
    std::fs::File::create(&file_with_path).expect("Couldn't make a file for some reason");
    open::that(file_with_path).unwrap();
}

fn main() -> Result<()> {
    let fragment: Fragment = argh::from_env();

    let main_window =
        WindowDesc::new(ui_builder).title(LocalizedString::new("").with_placeholder("Fragment"));

    let launcher = AppLauncher::with_window(main_window);

    let event_sink = launcher.get_external_handle();

    let data = FragmentState {
        results: Arc::new(search::list_of_all_files(
            &fragment.path,
            search::SortMethod::DateNewest,
        )),
        query: String::new(),
        path: fragment.path,
        event_sink: Arc::new(event_sink),
    };

    let _watch = data.watch();

    launcher.launch(data).unwrap();

    Ok(())
}

fn ui_builder() -> impl Widget<FragmentState> {
    Flex::column()
        .with_child(
            TextBox::new()
                .lens(FragmentState::query)
                .controller(KeyUp::new(|_, data: &mut FragmentState, _, key_event| {
                    if key_event.key_code == druid::KeyCode::Return {
                        data.query = data.query.trim().to_string();
                        create_note_and_open(&data.path, &data.query);
                    } else {
                        data.search();
                    }
                }))
                .expand_width()
                .padding(5.0),
        )
        .with_flex_child(
            Split::horizontal(
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
                        Scroll::new(
                            List::new(|| {
                                let painter = Painter::new(|ctx, _, env| {
                                    let bounds = ctx.size().to_rect();

                                    if ctx.is_hot() {
                                        ctx.fill(bounds, &env.get(theme::PRIMARY_DARK));
                                    }

                                    if ctx.is_active() {
                                        ctx.fill(bounds, &env.get(theme::PRIMARY_LIGHT));
                                    }
                                });

                                Flex::column().with_child(
                                    Flex::row()
                                        .with_flex_child(
                                            Label::new(|data: &ListItem, _: &Env| {
                                                (*data).file_name.to_string()
                                            })
                                            .padding(5.0)
                                            .expand_width(),
                                            2.0,
                                        )
                                        .with_flex_child(
                                            Label::new(|data: &ListItem, _: &Env| {
                                                let today = Utc::now().timestamp();
                                                let seconds_old =
                                                    data.modified.elapsed().unwrap().as_secs()
                                                        as i64;

                                                let date_modified = today - seconds_old;
                                                let dt: DateTime<Utc> =
                                                    Utc.timestamp(date_modified, 0);

                                                dt.format("%b %e, %Y").to_string()
                                            })
                                            .padding(5.0)
                                            .expand_width(),
                                            1.0,
                                        )
                                        .background(painter)
                                        .on_click(|_, data, _| {
                                            println!("hey");
                                            open_note_in_editor(&data.path)
                                        }),
                                )
                                // .with_child(Label::new(|data: &ListItem, _: &Env| {
                                //     (*data).line.to_string()
                                // }))
                            })
                            .expand_width()
                            .lens(FragmentState::results),
                        )
                        .vertical()
                        .expand_width(),
                        1.0,
                    )
                    .border(Color::rgb8(100, 100, 100), 1.0)
                    .rounded(5.0)
                    .padding(5.0),
                Scroll::new(
                    Flex::column()
                        .with_child(Label::new("Some day"))
                        .with_child(Label::new("we'll have"))
                        .with_child(Label::new("multiline text"))
                        .with_child(Label::new("and it's going to be so great"))
                        .with_child(Label::new("you just wait")),
                )
                .vertical()
                .expand_width(),
            )
            .draggable(true),
            1.0,
        )
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
        .controller(Search {
            phantom: std::marker::PhantomData::default(),
        })

    // .debug_paint_layout()
}
