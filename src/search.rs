use std::error::Error;
use std::ffi::OsString;
use std::io::prelude::*;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime};

use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};

use walkdir::{DirEntry, WalkDir};

use druid::{Data, Lens};

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
