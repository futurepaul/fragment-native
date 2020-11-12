use druid::{ExtEventSink, Target};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};

pub fn watch(path: String, event_sink: ExtEventSink) -> RecommendedWatcher {
    let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| match res {
        Ok(event) => {
            println!("event: {:?}", event);
            if let Err(_) =
                event_sink.submit_command(super::delegate::START_REFRESH, (), Target::Global)
            {
            }
        }
        Err(e) => println!("watch error: {:?}", e),
    })
    .expect("Couldn't create watcher");

    watcher
        .watch(
            std::path::Path::new(&path.clone()),
            RecursiveMode::Recursive,
        )
        .expect("Watcher couldn't watch");

    watcher
}
