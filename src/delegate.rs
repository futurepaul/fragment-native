use std::sync::Arc;

use druid::{AppDelegate, Command, DelegateCtx, Env, ExtEventSink, Handled, Selector, Target};

use super::{FragmentState, ListItem, Query};

pub const START_SEARCH: Selector = Selector::new("fragment.start-search");
pub const FINISH_SEARCH: Selector<Vec<ListItem>> = Selector::new("fragment.finish-search");
pub const START_REFRESH: Selector = Selector::new("fragment.refresh-search");
pub const LOAD_NOTE: Selector<String> = Selector::new("fragment.load-note");

pub struct Delegate {
    pub event_sink: ExtEventSink,
}

impl AppDelegate<FragmentState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut FragmentState,
        _env: &Env,
    ) -> Handled {
        if cmd.is(START_SEARCH) {
            data.sender
                .send(Query {
                    query: data.query.clone(),
                    event_sink: self.event_sink.clone(),
                })
                .unwrap();
            Handled::Yes
        } else if let Some(search_result) = cmd.get(FINISH_SEARCH) {
            data.results = Arc::new(search_result.clone());
            Handled::Yes
        } else if cmd.is(START_REFRESH) {
            data.sender
                .send(Query {
                    query: data.query.clone(),
                    event_sink: self.event_sink.clone(),
                })
                .unwrap();
            Handled::Yes
        } else if let Some(note) = cmd.get(LOAD_NOTE) {
            data.selected_note = Some(note.to_string());
            Handled::Yes
        } else {
            Handled::No
        }
    }
}
