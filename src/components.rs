use chrono::prelude::*;

use druid::widget::{Either, Flex, Label, LineBreaking, List, Painter, Scroll, TextBox, WidgetExt};
use druid::{theme, Color, Env, RenderContext, Widget};

use super::keyup::KeyUp;
use super::ListItem;

use super::FragmentState;

fn list_item() -> impl Widget<ListItem> {
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
        .on_click(ListItem::preview_note)
}

pub(crate) fn top_pane() -> impl Widget<FragmentState> {
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
            Scroll::new(List::new(|| list_item()).lens(FragmentState::results))
                .vertical()
                .expand_width(),
            1.0,
        )
        .border(Color::rgb8(100, 100, 100), 1.0)
        .rounded(5.0)
        .padding(5.0)
}

pub(crate) fn search_box() -> impl Widget<FragmentState> {
    TextBox::new()
        .lens(FragmentState::query)
        .controller(KeyUp::new())
        .expand_width()
        .padding(5.0)
}

pub(crate) fn text_pane() -> impl Widget<FragmentState> {
    let text = Either::new(
        |data: &FragmentState, env: &Env| data.selected_note.is_some(),
        Label::dynamic(|data: &FragmentState, env: &Env| {
            data.selected_note.clone().unwrap_or("wtf".to_string())
        })
        .with_line_break_mode(LineBreaking::WordWrap),
        Label::new(
            "Some day we'll have multiline text and it's going to be so great just you wait",
        )
        .with_line_break_mode(LineBreaking::WordWrap),
    );
    Scroll::new(
        Flex::column()
            .with_child(text)
            // .with_child(Label::new("we'll have"))
            // .with_child(Label::new("multiline text"))
            // .with_child(Label::new("and it's going to be so great"))
            // .with_child(Label::new("you just wait")),
    )
    .vertical()
    .expand_width()
}
