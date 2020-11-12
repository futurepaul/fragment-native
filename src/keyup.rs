use std::marker::PhantomData;

use druid::widget::Controller;
use druid::{Env, Event, EventCtx, HotKey, KbKey, Widget};

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
            Event::KeyUp(key_event) => match key_event {
                k_e if (HotKey::new(None, KbKey::Enter)).matches(k_e) => {
                    data.query = data.query.trim().to_string();
                    data.create_note_and_open()
                        .expect("couldn't create note and open");
                }
                _ => {
                    ctx.submit_command(super::delegate::START_SEARCH);
                    child.event(ctx, event, data, env);
                }
            },
            _ => child.event(ctx, event, data, env),
        }
    }
}
