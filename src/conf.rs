use std::error::Error;
use std::fs;
use std::path::Path;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::clone::GitRepo;

// Todo: This will never be serialized. Try removing Serialize.
#[derive(Debug, Serialize, Deserialize)]
pub struct GGConf {
    #[serde(alias = "skipDirectories")]
    #[serde(rename = "skipDirectories")]
    #[serde(default)]
    filter_list: Vec<String>,
    #[serde(skip)]
    pub filter_list_regex: Vec<Regex>,
    #[serde(alias = "cloneRepos")]
    #[serde(rename = "cloneRepos")]
    #[serde(default)]
    // Todo: Add validations on this field. It should not allow empty key/values.
    pub clone_repos: Vec<GitRepo>,
}

impl GGConf {
    pub fn default() -> GGConf {
        GGConf {
            filter_list: vec![],
            filter_list_regex: vec![],
            clone_repos: vec![],
        }
    }
}

pub fn read_conf_file(conf_file: &str) -> Result<GGConf, Box<dyn Error>> {
    if Path::new(conf_file).exists() {
        let file = fs::File::open(conf_file)?;
        let mut config: GGConf = serde_yaml::from_reader(file)?;
        update_conf_file(&mut config)?;
        return Ok(config);
    }
    let mut default = GGConf::default();
    update_conf_file(&mut default)?;
    Ok(default)
}

fn update_conf_file(conf: &mut GGConf) -> Result<&mut GGConf, Box<dyn Error>> {
    create_filter_list(conf)?;
    Ok(conf)
}

fn create_filter_list(conf: &mut GGConf) -> Result<&mut GGConf, Box<dyn Error>> {
    let mut filter_list = Vec::new();
    let mut filters = conf.filter_list.clone();
    let defaults: Vec<String> = [".idea", ".DS_Store"].iter().map(|&s| s.into()).collect();
    defaults.iter().for_each(|def| {
        filters.push(def.to_owned());
    });

    filters.iter().for_each(|ignore| {
        let re = Regex::new(format!(r".*/{}?*", ignore).as_str()).expect("failed to construct regex");
        filter_list.push(re);
    });

    conf.filter_list = filters;
    conf.filter_list_regex = filter_list;
    Ok(conf)
}
