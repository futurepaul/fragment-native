use druid::widget::{Button, Checkbox, Flex, Label, List, Painter, Scroll, TextBox, WidgetExt};
use druid::{
    theme, AppLauncher, Color, Data, Env, Lens, LocalizedString, PlatformError, RenderContext,
    Widget, WindowDesc,
};
use std::mem;
use std::sync::Arc;

use open;

mod search;
use search::ListItem;

// const COUNT_BG: Color = Color::grey8(0x77);

#[derive(Debug, Clone, Data, Default, Lens)]
struct FragmentState {
    results: Arc<Vec<ListItem>>,
    query: String,
}

impl FragmentState {
    // fn add_item(&mut self, text: impl Into<String>) {
    //     let text = text.into();
    //     let item = FragmentItem { text };
    //     Arc::make_mut(&mut self.results).push(item);
    // }
    fn search(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.results = Arc::new(search::search(&self.query, "./notes").unwrap());
    }
}

// fn dummy_data() -> FragmentState {
//     let mut list = FragmentState::default();
//     list.add_item("buy eggs");
//     list.add_item("buy shoes");
//     list.add_item("never die");
//     list
// }

fn open_note_in_editor(path: &str) {
    open::that(path).unwrap();
}

fn main() -> Result<(), PlatformError> {
    // let search_results = search("fn", ".").unwrap();

    // dbg!(search_results);

    // let mut list = FragmentState::default();

    // for result in search_results {
    //     list.add_item("buy eggs");
    //     list.add_item("buy shoes");
    //     list.add_item("never die");
    // }

    let mut data = FragmentState::default();
    data.search("fn");

    let main_window =
        WindowDesc::new(ui_builder).title(LocalizedString::new("").with_placeholder("Fragment"));
    AppLauncher::with_window(main_window)
        // .use_simple_logger()
        .launch(data)?;

    Ok(())
}

fn ui_builder() -> impl Widget<FragmentState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_flex_child(
                    TextBox::new()
                        .expand_width()
                        .padding(5.0)
                        .lens(FragmentState::query),
                    1.0,
                )
                .with_child(
                    Button::new("Search", |_ctx, data: &mut FragmentState, _| {
                        let item_text = mem::take(&mut data.query);
                        data.search(item_text);
                    })
                    .padding(5.0),
                ),
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
                        Label::new(|data: &ListItem, _env: &Env| data.file_name.clone())
                            .expand_width()
                            .background(painter)
                            .on_click(|_, data, _| {
                                println!("hey");
                                open_note_in_editor(&data.path)
                            }),
                        1.0,
                    )
                })
                .expand_width()
                .padding(5.0)
                .lens(FragmentState::results),
            )
            .vertical()
            .expand_width(),
            1.0,
        )
        .cross_axis_alignment(druid::widget::CrossAxisAlignment::Start)
    // .debug_paint_layout()
}
