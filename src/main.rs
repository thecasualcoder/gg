extern crate clap;
extern crate colored;
extern crate git2;
extern crate walkdir;

use clap::{App, AppSettings, SubCommand, Arg};

mod input_args;
mod status;

fn main() {
    let app = App::new("Git Governance")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version("1.0")
        .subcommand(SubCommand::with_name("status")
            .arg(Arg::with_name("PATH")
                .short("f")
                .takes_value(true)
                .help("path of the directory from which the git governance will start analysing the git repos")
            )
        );

    let args = input_args::InputArgs::parse_inputs(app.get_matches());

    match args.input_command() {
        input_args::InputCommand::Status => status::status(args.get_matches()),
        input_args::InputCommand::Create => create(),
        input_args::InputCommand::Error => {}
    }
}

fn create() {
//    Todo: to be implemented
}
