use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use argh::FromArgs;
use crossbeam_channel::Sender;
use open;

use druid::widget::{Flex, Split};
use druid::{AppLauncher, Data, ExtEventSink, Lens, LocalizedString, Widget, WindowDesc};

mod components;
mod delegate;
mod watch;

mod search;
use search::ListItem;

mod keyup;

#[derive(Debug)]
pub enum FragmentError {
    Io(std::io::Error),
    Druid(druid::PlatformError),
    DruidExtEvent(druid::ExtEventError),
}

#[derive(FromArgs)]
/// Search notes.
struct FragmentArgs {
    /// path of the notes folder to search
    #[argh(option, short = 'p')]
    path: String,
}

#[derive(Clone, Data, Lens)]
struct FragmentState {
    results: Arc<Vec<ListItem>>,
    query: String,
    path: String,
    sender: Arc<Sender<Query>>,
}

pub struct Query {
    query: String,
    event_sink: ExtEventSink,
}

impl FragmentState {
    fn new(path: String) -> FragmentState {
        let initial_results = search::search("", &path.clone(), &AtomicU64::new(0), 1)
            .expect("Initial search failed");

        FragmentState {
            results: Arc::new(initial_results),
            query: String::new(),
            path: path.clone(),
            sender: search::spawn_search_thread(path),
        }
    }

    fn create_note_and_open(&self) -> Result<(), FragmentError> {
        let file_with_path = std::path::Path::new(&self.path)
            .join(self.query.clone())
            .with_extension("md");
        std::fs::File::create(&file_with_path).map_err(FragmentError::Io)?;
        open::that(file_with_path).map_err(FragmentError::Io)?;
        Ok(())
    }
}

fn main() -> Result<(), FragmentError> {
    // Get path argument from command line
    let args: FragmentArgs = argh::from_env();
    let path = args.path;

    let main_window =
        WindowDesc::new(ui_builder).title(LocalizedString::new("").with_placeholder("Fragment"));
    let launcher = AppLauncher::with_window(main_window);
    let event_sink = launcher.get_external_handle();

    let delegate = delegate::Delegate {
        event_sink: event_sink.clone(),
    };

    // Fire up a thread to notify of changes at the root path
    let _watch = watch::watch(path.clone(), event_sink.clone());

    launcher
        .delegate(delegate)
        .launch(FragmentState::new(path))
        .map_err(FragmentError::Druid)?;

    Ok(())
}

fn ui_builder() -> impl Widget<FragmentState> {
    Flex::column()
        // Search box. Automatically gains focus on launch
        .with_child(components::search_box())
        // The rest of the app
        .with_flex_child(
            Split::horizontal(
                // Search results
                components::top_pane(),
                // File preview (TODO)
                components::text_pane(),
            )
            .split_point(0.8)
            .draggable(true),
            1.0,
        )
}
