use argh::FromArgs;
use druid::widget::{
    Button, Checkbox, Container, Flex, Label, List, MainAxisAlignment, Painter, Scroll, Split,
    TextBox, WidgetExt,
};
use druid::{
    theme, AppLauncher, Color, Data, Env, ExtEventSink, Lens, LocalizedString, PlatformError,
    RenderContext, Selector, Widget, WindowDesc,
};
use std::mem;
use std::sync::Arc;
use std::thread;

use open;

mod search;
use search::ListItem;

mod keyup;
use keyup::KeyUp;

mod search_controller;
use search_controller::{Search, SEARCH_RESULTS};

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
    fn search(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.results = Arc::new(search::search(&self.query, &self.path).unwrap());
    }
}

fn search(query: String, path: String, event_sink: Arc<ExtEventSink>) {
    thread::spawn(move || {
        // if this fails we're shutting down
        let results = search::search(&query, &path).unwrap();
        if let Err(_) = event_sink.submit_command(SEARCH_RESULTS, results, None) {}
    });
}

fn open_note_in_editor(path: &str) {
    open::that(path).unwrap();
}

fn main() -> Result<(), PlatformError> {
    let fragment: Fragment = argh::from_env();

    // data.search("fn");

    let main_window =
        WindowDesc::new(ui_builder).title(LocalizedString::new("").with_placeholder("Fragment"));

    let launcher = AppLauncher::with_window(main_window);
    // .use_simple_logger()

    let event_sink = launcher.get_external_handle();

    let data = FragmentState {
        results: Arc::new(vec![]),
        query: String::new(),
        path: fragment.path,
        event_sink: Arc::new(event_sink),
    };

    launcher.launch(data)?;

    Ok(())
}

fn ui_builder() -> impl Widget<FragmentState> {
    Flex::column()
        .with_child(
            TextBox::new()
                .lens(FragmentState::query)
                .controller(KeyUp::new(|_, data: &mut FragmentState, _, key_event| {
                    let item_text = data.query.clone();
                    // data.search(item_text);
                    search(item_text, data.path.clone(), data.event_sink.clone());
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

                                Flex::row().with_flex_child(
                                    Flex::row()
                                        .with_flex_child(
                                            Label::new(|data: &ListItem, _env: &Env| {
                                                data.file_name.clone()
                                            })
                                            .padding(5.0)
                                            .expand_width(),
                                            2.0,
                                        )
                                        .with_flex_child(
                                            Label::new("Today 1:58PM").padding(5.0).expand_width(),
                                            1.0,
                                        )
                                        .background(painter)
                                        .on_click(|_, data, _| {
                                            println!("hey");
                                            open_note_in_editor(&data.path)
                                        }),
                                    1.0,
                                )
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
                        .with_child(Label::new("you just wait"))
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
