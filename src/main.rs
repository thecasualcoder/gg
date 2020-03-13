#![recursion_limit = "128"]
extern crate clap;
extern crate colored;
extern crate git2;
extern crate reqwest;
extern crate serde_yaml;
extern crate walkdir;

use std::process;

use clap::{crate_version, App, AppSettings, Arg};
use colored::*;

mod clone;
mod conf;
mod create;
mod dir;
mod fetch;
mod git;
mod input_args;
mod status;

mod progress;

fn main() {
    let app = App::new("Git Governance")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(crate_version!())
        .arg(
            Arg::with_name("conf")
                .short("c")
                .global(true)
                .takes_value(true)
                .help("config file to use. Defaults to .ggConf"),
        )
        .subcommand(status::sub_command())
        .subcommand(create::sub_command())
        .subcommand(fetch::sub_command())
        .subcommand(clone::sub_command());

    let global_matches = app.get_matches();

    let args = input_args::InputArgs::parse_inputs(global_matches.clone());

    let conf = conf::read_conf_file(global_matches.value_of("conf").unwrap_or(".ggConf.yaml"))
        .unwrap_or_else(|err| {
            println!(
                "{} {}",
                "error while reading conf file:".red(),
                err.to_string().red()
            );
            process::exit(1)
        });

    match args.input_command() {
        input_args::InputCommand::Status => status::status(args, conf.filter_list_regex),
        input_args::InputCommand::Create => create::create(args),
        input_args::InputCommand::Clone => clone::clone(args, conf.clone_repos),
        input_args::InputCommand::Fetch => fetch::fetch(args, conf.filter_list_regex),
        input_args::InputCommand::Error => {}
    }
}
