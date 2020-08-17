#![recursion_limit = "128"]
extern crate clap;
extern crate colored;
extern crate git2;
extern crate lazy_static;
extern crate reqwest;
extern crate serde_yaml;
extern crate walkdir;

use std::{mem, sync::Mutex};
use std::process;

use clap::{crate_version, App, AppSettings, Arg};
use colored::*;
use lazy_static::lazy_static;

use crate::conf::SSHConfig;

mod clone;
mod branches;
mod conf;
mod create;
mod dir;
mod fetch;
mod git;
mod input_args;
mod status;

mod progress;

lazy_static! {
   pub static ref SSH_CONF: Mutex<SSHConfig> =  Mutex::new(SSHConfig{
        private_key: String::from(format!("{}/.ssh/id_rsa", std::env::var("HOME").expect("HOME env not found"))),
        ssh_agent: false,
        username: String::from("git"),
   });
}

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
        .arg(
            Arg::with_name("jobs")
                .short("j")
                .global(true)
                .takes_value(true)
                .help("Specifies the number of jobs to run simultaneously.\
                Set to 1 to go monothread, by default is set to your number of CPUs.")
                .validator(|str| {
                    str.parse()
                        .map(|_: usize| ())
                        .map_err(|err| format!("{}", err))
                }),
        )
        .subcommand(status::sub_command())
        .subcommand(create::sub_command())
        .subcommand(fetch::sub_command())
        .subcommand(branches::sub_command())
        .subcommand(clone::sub_command());


    let global_matches = app.get_matches();

    let args = input_args::InputArgs::parse_inputs(global_matches.clone());

    let conf = conf::read_conf_file(global_matches.value_of("conf").unwrap_or(".ggConf.yaml"))
        .unwrap_or_else(|err| {
            println!("{} {}", "error while reading conf file:".red(), err.to_string().red());
            process::exit(1)
        });

    if conf.ssh_config.is_some() {
        mem::replace(
            &mut *SSH_CONF.lock().unwrap(),
            conf.ssh_config.unwrap(),
        );
    }

    match args.input_command() {
        input_args::InputCommand::Status => status::status(args, conf.filter_list_regex),
        input_args::InputCommand::Create => create::create(args),
        input_args::InputCommand::Branches => branches::branches(args),
        input_args::InputCommand::Clone => clone::clone(args, conf.clone_repos),
        input_args::InputCommand::Fetch => fetch::fetch(args, conf.filter_list_regex),
        input_args::InputCommand::Error => {}
    }
}
