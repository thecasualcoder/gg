#![recursion_limit = "128"]
extern crate reqwest;
extern crate clap;
extern crate colored;
extern crate git2;
extern crate walkdir;

use clap::{crate_version, App, AppSettings};
use regex::Regex;
use std::error::Error;

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
        .subcommand(status::sub_command())
        .subcommand(create::sub_command())
        .subcommand(fetch::sub_command());

    let args = input_args::InputArgs::parse_inputs(app.get_matches());

    let filter_list = create_filter_list().expect("failed to create filter_list");

    match args.input_command() {
        input_args::InputCommand::Status => status::status(args, filter_list),
        input_args::InputCommand::Create => create::create(args.get_matches()),
        input_args::InputCommand::Fetch => fetch::fetch(args, filter_list),
        input_args::InputCommand::Error => {}
    }
}


fn create_filter_list() -> Result<Vec<Regex>, Box<dyn Error>> {
    // Todo: The filter list is hard coded for now. This should be set from a file.
    // Todo: Sensible defaults can be added to it in code(.git, .DS_STORE, .idea, a lot of dot directories).
    let mut filter_list = Vec::new();
    let re = Regex::new(r"^ignore/*")?;
    filter_list.push(re);
    Ok(filter_list)
}
