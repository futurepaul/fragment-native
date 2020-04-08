use druid::widget::Controller;
use druid::{Data, Env, Event, EventCtx, KeyEvent, LifeCycle, LifeCycleCtx, Widget};

pub struct KeyUp<T> {
    action: Box<dyn Fn(&mut EventCtx, &mut T, &Env, &KeyEvent)>,
}

impl<T: Data> KeyUp<T> {
    pub fn new(action: impl Fn(&mut EventCtx, &mut T, &Env, &KeyEvent) + 'static) -> Self {
        KeyUp {
            action: Box::new(action),
        }
    }
}

impl<T: Data, W: Widget<T>> Controller<T, W> for KeyUp<T> {
    fn event(&mut self, child: &mut W, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::WindowConnected => {
                ctx.request_focus();
            }
            Event::KeyUp(key_event) => {
                (self.action)(ctx, data, env, key_event);
            }
            _ => {}
        }

        child.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(_) | LifeCycle::FocusChanged(_) = event {
            ctx.request_paint();
        }

        child.lifecycle(ctx, event, data, env);
    }
}
