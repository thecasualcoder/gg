#![recursion_limit = "128"]
extern crate reqwest;
extern crate clap;
extern crate colored;
extern crate git2;
extern crate walkdir;

use clap::{App, AppSettings};

mod input_args;
mod status;
mod create;
mod fetch;
mod git;

fn main() {
    let app = App::new("Git Governance")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("1.0")
        .subcommand(status::sub_command())
        .subcommand(create::sub_command())
        .subcommand(fetch::sub_command());

    let args = input_args::InputArgs::parse_inputs(app.get_matches());

    match args.input_command() {
        input_args::InputCommand::Status => status::status(args.get_matches()),
        input_args::InputCommand::Create => create::create(args.get_matches()),
        input_args::InputCommand::Fetch => fetch::fetch(args.get_matches()),
        input_args::InputCommand::Error => {}
    }
}
