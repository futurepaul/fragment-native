use druid::widget::Controller;
use druid::{Data, Env, Event, EventCtx, KeyEvent, LifeCycle, LifeCycleCtx, Selector, Widget};

use super::{FragmentState, ListItem};
use std::sync::Arc;

pub const SEARCH_RESULTS: Selector = Selector::new("fragment.search-results");
pub struct Search<FragmentState> {
    pub phantom: std::marker::PhantomData<FragmentState>,
}

impl<W: Widget<FragmentState>> Controller<FragmentState, W> for Search<FragmentState> {
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut FragmentState,
        env: &Env,
    ) {
        match event {
            Event::Command(cmd) if cmd.selector == SEARCH_RESULTS => {
                data.results = Arc::new(cmd.get_object::<Vec<ListItem>>().unwrap().clone());
                ctx.request_paint();
            }
            _ => (),
        }

        child.event(ctx, event, data, env);
    }

    fn lifecycle(
        &mut self,
        child: &mut W,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &FragmentState,
        env: &Env,
    ) {
        // if let LifeCycle::HotChanged(_) | LifeCycle::FocusChanged(_) = event {
        //     ctx.request_paint();
        // }

        child.lifecycle(ctx, event, data, env);
    }
}
