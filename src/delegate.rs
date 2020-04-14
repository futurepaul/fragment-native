use std::sync::Arc;

use druid::{AppDelegate, Command, DelegateCtx, Env, ExtEventSink, Selector, Target};

use super::{FragmentState, ListItem, Query};

pub const START_SEARCH: Selector = Selector::new("fragment.start-search");
pub const FINISH_SEARCH: Selector = Selector::new("fragment.finish-search");
pub const START_REFRESH: Selector = Selector::new("fragment.refresh-search");

pub struct Delegate {
    pub event_sink: ExtEventSink,
}

impl AppDelegate<FragmentState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: &Target,
        cmd: &Command,
        data: &mut FragmentState,
        _env: &Env,
    ) -> bool {
        match cmd.selector {
            START_SEARCH => {
                data.sender
                    .send(Query {
                        query: data.query.clone(),
                        event_sink: self.event_sink.clone(),
                    })
                    .unwrap();
                false
            }
            FINISH_SEARCH => {
                data.results = Arc::new(
                    cmd.get_object::<Vec<ListItem>>()
                        .expect("Couldn't get_object")
                        .clone(),
                );
                false
            }
            START_REFRESH => {
                data.sender
                    .send(Query {
                        query: data.query.clone(),
                        event_sink: self.event_sink.clone(),
                    })
                    .unwrap();
                false
            }
            _ => true,
        }
    }
}
