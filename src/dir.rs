use std::error::Error;

use regex::Regex;
use walkdir::DirEntry;

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

    pub fn should_filter(&self, entry: &DirEntry) -> Result<bool, Box<dyn Error>> {
        let path_string = entry.path().to_str().expect("could not get path from the entry").trim_start_matches("./");

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
