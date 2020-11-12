use std::ffi::OsString;
use std::io::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::{error::Error, fs::File};

use std::thread;
use std::time::{Instant, SystemTime};

use crossbeam_channel::Sender;

use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};

use walkdir::{DirEntry, WalkDir};

use druid::{Command, Data, Env, EventCtx, Lens, Target};

use crate::delegate::LOAD_NOTE;

use super::Query;

#[derive(Clone, Debug, Data, Lens)]
pub struct ListItem {
    pub path: Arc<str>,
    pub file_name: Arc<str>,
    #[data(same_fn = "PartialEq::eq")]
    pub modified: SystemTime,
    pub first_line: Arc<str>,
    pub found_line: Option<Arc<str>>,
}

impl ListItem {
    pub fn open_note_in_editor(&self) {
        open::that(self.path.as_ref()).expect("Couldn't open file");
    }
    pub fn preview_note(ctx: &mut EventCtx, data: &mut ListItem, env: &Env) {
        let mut file = File::open(data.path.as_ref()).expect("Couldn't open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Couldn't read file to string");
        ctx.submit_command(Command::new(LOAD_NOTE, contents, Target::Global))
    }
}

pub enum SortMethod {
    DateNewest,
    // DateOldest,
    // TitleAZ,
    // TitleZA,
    // NoSort,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn spawn_search_thread(path: String) -> Arc<Sender<Query>> {
    let (s, r) = crossbeam_channel::bounded::<Query>(1);

    let atomic = Arc::new(AtomicU64::new(0));

    thread::spawn(move || loop {
        match r.recv() {
            Ok(sender_query) => {
                let query = sender_query.query.clone();
                let event_sink = sender_query.event_sink.clone();

                let path = path.clone();
                let atomic = atomic.clone();

                thread::spawn(move || {
                    let results = search(&query, &path, &atomic, atomic.load(Ordering::SeqCst) + 1)
                        .expect("Search failed");
                    if let Err(_) = event_sink.submit_command(
                        super::delegate::FINISH_SEARCH,
                        results,
                        Target::Global,
                    ) {};
                });
            }
            Err(e) => println!("Receive error: {:?}", e),
        };
    });

    Arc::new(s)
}

pub fn search(
    pattern: &str,
    dir: &str,
    sequence_ref: &AtomicU64,
    self_sequence: u64,
) -> Result<Vec<ListItem>, Box<dyn Error>> {
    sequence_ref.fetch_add(1, Ordering::SeqCst);

    let files = list_of_all_files(dir, SortMethod::DateNewest, sequence_ref, self_sequence);

    grep_life(pattern, &files, sequence_ref, self_sequence)
}

fn first_line(path: &str) -> String {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(_) => panic!("Unable to read title from {}", path),
    };
    let mut buffer = std::io::BufReader::new(file);
    let mut first_line = String::new();

    match buffer.read_line(&mut first_line) {
        Ok(_) => {}
        Err(_e) => {
            eprintln!("Couldn't read file: {}", path);
        }
    }

    first_line
}

pub fn list_of_all_files(
    root: &str,
    sort_by: SortMethod,
    sequence_ref: &AtomicU64,
    self_sequence: u64,
) -> Vec<ListItem> {
    let list_start = Instant::now();
    let dir = OsString::from(root);

    let mut list = Vec::new();

    let walker = WalkDir::new(dir).into_iter();
    for result in walker.filter_entry(|e| !is_hidden(e)) {
        if sequence_ref.load(Ordering::SeqCst) > self_sequence {
            eprintln!(
                "List files ref: {}, mine: {}",
                sequence_ref.load(Ordering::SeqCst),
                self_sequence
            );
            eprintln!("List files ended early!");
            break;
        }

        match result {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    list.push(ListItem {
                        path: entry.path().display().to_string().into(),
                        file_name: entry
                            .file_name()
                            .to_os_string()
                            .into_string()
                            .expect("Couldn't convert into string.")
                            .into(),
                        modified: get_modified_time_from_path(&entry.path().display().to_string()),
                        first_line: first_line(&entry.path().display().to_string())
                            .trim()
                            .into(),
                        found_line: None,
                    })
                }
            }
            Err(err) => println!("WALKDIR ERROR: {}", err),
        }
    }

    match sort_by {
        SortMethod::DateNewest => list.sort_unstable_by(|a, b| b.modified.cmp(&a.modified)),
        // SortMethod::DateOldest => list.sort_unstable_by(|a, b| a.modified.cmp(&b.modified)),
        // SortMethod::TitleAZ => list.sort_unstable_by(|a, b| a.file_name.cmp(&b.file_name)),
        // SortMethod::TitleZA => list.sort_unstable_by(|a, b| b.file_name.cmp(&a.file_name)),
        // SortMethod::NoSort => {}
    }

    let list_end = Instant::now();
    println!("list files took: {}ms", (list_end - list_start).as_millis());

    list
}

fn get_modified_time_from_path(path: &str) -> SystemTime {
    match Path::new(path).metadata() {
        Ok(metadata) => metadata
            .modified()
            .expect("What to do if this doesn't work?"),
        Err(_e) => panic!("I don't know what to do if we don't have metadata"),
    }
}

pub fn grep_life(
    pattern: &str,
    files: &Vec<ListItem>,
    sequence_ref: &AtomicU64,
    self_sequence: u64,
) -> Result<Vec<ListItem>, Box<dyn Error>> {
    let grep_start = Instant::now();

    let mut matches: Vec<ListItem> = vec![];
    let matcher = RegexMatcher::new(&pattern)?;
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();

    for file in files {
        if sequence_ref.load(Ordering::SeqCst) > self_sequence {
            eprintln!(
                "Grep ref: {}, mine: {}",
                sequence_ref.load(Ordering::SeqCst),
                self_sequence
            );
            eprintln!("Grep ended early!");
            break;
        }
        let result = searcher.search_path(
            &matcher,
            &file.path.to_string(),
            UTF8(|ln, line| {
                matches.push(ListItem {
                    path: file.path.clone(),
                    file_name: file.file_name.clone(),
                    modified: file.modified,
                    first_line: file.first_line.clone(),
                    found_line: Some(format!("{}: {}", ln, line.trim()).into()),
                });
                //we stop searching after our first find by returning false
                Ok(false)
            }),
        );
        if let Err(err) = result {
            println!("GREP ERROR: {}: {}", file.path, err);
        }
    }

    let grep_end = Instant::now();
    println!("grep took: {}ms", (grep_end - grep_start).as_millis());
    Ok(matches)
}
