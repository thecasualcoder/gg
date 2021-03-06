use std::error::Error;

use regex::Regex;
use walkdir::{DirEntry, WalkDir};

pub struct DirectoryTreeOptions {
    pub filter_list: Vec<Regex>,
    pub filter_hidden: bool,
}

impl DirectoryTreeOptions {
    fn is_not_hidden(&self, entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|file| entry.depth() == 0 || !file.starts_with("."))
            .unwrap_or(false)
    }

    pub fn process_directories(self, path: &str) -> impl Iterator<Item=Result<DirEntry, walkdir::Error>> {
        WalkDir::new(path)
            .follow_links(false)
            .contents_first(false)
            .same_file_system(true)
            .into_iter()
            .filter_entry(move |e| {
                self.should_filter(e)
                    .expect(format!("failed to filter for entry {:#?}", e).as_str())
            })
    }

    fn should_filter(&self, entry: &DirEntry) -> Result<bool, Box<dyn Error>> {
        let path_string = entry
            .path()
            .to_str()
            .expect("could not get path from the entry")
            .trim_start_matches("./");

        if self.filter_hidden {
            return Ok(self.is_not_hidden(entry));
        }

        if !entry.path().is_dir() {
            return Ok(false);
        }

        for ignore in self.filter_list.iter() {
            if ignore.is_match(path_string) {
                return Ok(false);
            }
        }

        return Ok(true);
    }
}
