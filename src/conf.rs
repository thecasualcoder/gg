use std::error::Error;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
pub struct GGConf {
    #[serde(alias = "skipDirectories")]
    #[serde(rename = "skipDirectories")]
    filter_list: Vec<String>,
    #[serde(skip)]
    pub filter_list_regex: Vec<Regex>,
}

pub fn read_conf_file(conf_file: &str) -> Result<GGConf, Box<dyn Error>> {
    if Path::new(conf_file).exists() {
        let file = fs::File::open(conf_file)?;
        let config: GGConf = serde_yaml::from_reader(file)?;
        let updated_conf = create_filter_list(config)?;
        return Ok(updated_conf);
    }
    let default = GGConf { filter_list: vec![], filter_list_regex: vec![] };
    let updated_conf = create_filter_list(default)?;
    Ok(updated_conf)
}

fn create_filter_list(conf: GGConf) -> Result<GGConf, Box<dyn Error>> {
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

    let updated_conf = GGConf { filter_list: conf.filter_list, filter_list_regex: filter_list };
    Ok(updated_conf)
}
