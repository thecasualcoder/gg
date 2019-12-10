use std::error::Error;

use regex::Regex;
use walkdir::{DirEntry};

pub struct DirectoryTreeOptions {
    pub filter_list: Vec<Regex>,
    pub filter_hidden: bool,
}

impl DirectoryTreeOptions {
    pub fn should_filter(&self, entry: &DirEntry) -> Result<bool, Box<dyn Error>> {
        let path_string = entry.path().to_str().expect("could not get path from the entry").trim_start_matches("./");

        if self.filter_hidden && path_string.len() > 1 && path_string.starts_with(".") {
            return Ok(false);
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
