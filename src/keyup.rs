use std::marker::PhantomData;

use druid::widget::Controller;
use druid::{Env, Event, EventCtx, KeyCode, Widget};

use super::FragmentState;

pub struct KeyUp<FragmentState> {
    phantom: PhantomData<FragmentState>,
}

impl KeyUp<FragmentState> {
    pub fn new() -> KeyUp<FragmentState> {
        KeyUp {
            phantom: PhantomData::default(),
        }
    }
}

impl<W: Widget<FragmentState>> Controller<FragmentState, W> for KeyUp<FragmentState> {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut FragmentState,
        env: &Env,
    ) {
        match event {
            Event::WindowConnected => {
                ctx.request_focus();
            }
            // TODO: some reason I'm still getting this flash of a box character
            Event::KeyUp(k) if k.key_code != KeyCode::Return => {
                ctx.submit_command(super::delegate::START_SEARCH, None);
                child.event(ctx, event, data, env);
            }
            Event::KeyUp(k) if k.key_code == KeyCode::Return => {
                data.query = data.query.trim().to_string();
                data.create_note_and_open()
                    .expect("couldn't create note and open");
            }
            _ => child.event(ctx, event, data, env),
        }
    }
}
