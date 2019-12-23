#![recursion_limit = "128"]
extern crate clap;
extern crate colored;
extern crate git2;
extern crate reqwest;
extern crate serde_yaml;
extern crate walkdir;

use std::{fs, process};
use std::error::Error;
use std::path::Path;

use clap::{App, AppSettings, Arg, crate_version};
use colored::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

mod input_args;
mod status;
mod create;
mod fetch;
mod git;
mod dir;

fn main() {
    let app = App::new("Git Governance")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .arg(Arg::with_name("conf")
            .short("c")
            .global(true)
            .takes_value(true)
            .help("config file to use. Defaults to .ggConf"))
        .subcommand(status::sub_command())
        .subcommand(create::sub_command())
        .subcommand(fetch::sub_command());


    let global_matches = app.get_matches();

    let args = input_args::InputArgs::parse_inputs(global_matches.clone());

    let conf = read_conf_file(global_matches.value_of("conf").unwrap_or(".ggConf.yaml")).unwrap_or_else(|err| {
        println!("{} {}", "error while reading conf file:".red(), err.to_string().red());
        process::exit(1)
    });

    match args.input_command() {
        input_args::InputCommand::Status => {
            let filter_list = create_filter_list(conf).expect("failed to create filter_list");
            status::status(args, filter_list)
        }
        input_args::InputCommand::Create => create::create(args),
        input_args::InputCommand::Fetch => {
            let filter_list = create_filter_list(conf).expect("failed to create filter_list");
            fetch::fetch(args, filter_list)
        }
        input_args::InputCommand::Error => {}
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GGConf {
    #[serde(alias = "skipDirectories")]
    #[serde(rename = "skipDirectories")]
    filter_list: Vec<String>,
}

fn read_conf_file(conf_file: &str) -> Result<GGConf, Box<dyn Error>> {
    if Path::new(conf_file).exists() {
        let file = fs::File::open(conf_file)?;
        let config: GGConf = serde_yaml::from_reader(file)?;
        return Ok(config);
    }
    let default = GGConf { filter_list: vec![] };
    Ok(default)
}

fn create_filter_list(conf: GGConf) -> Result<Vec<Regex>, Box<dyn Error>> {
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


    Ok(filter_list)
}
