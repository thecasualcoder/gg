use clap::ArgMatches;
use std::path::{PathBuf, Path};
use colored::*;
use std::env::current_dir;
use std::process;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputCommand {
    Status,
    Create,
    Fetch,
    Error,
}

impl InputCommand {
    fn as_str(&self) -> &'static str {
        match *self {
            InputCommand::Status => "status",
            InputCommand::Create => "create",
            InputCommand::Fetch => "fetch",
            _ => "unknown command"
        }
    }
}

pub struct InputArgs<'a> {
    input_command: InputCommand,
    arg_matches: ArgMatches<'a>,
}

impl<'a> InputArgs<'a> {
    pub fn parse_inputs(args: ArgMatches) -> InputArgs {
        let subcommand_name = args.subcommand_name().expect("Could not get subcommand name");
        let matches = args.subcommand_matches(subcommand_name).expect("Failed to get arg matches");
        if subcommand_name == InputCommand::Status.as_str() {
            InputArgs {
                input_command: InputCommand::Status,
                arg_matches: matches.to_owned(),
            }
        } else if subcommand_name == InputCommand::Create.as_str() {
            InputArgs {
                input_command: InputCommand::Create,
                arg_matches: matches.to_owned(),
            }
        } else if subcommand_name == InputCommand::Fetch.as_str() {
            InputArgs {
                input_command: InputCommand::Fetch,
                arg_matches: matches.to_owned(),
            }
        } else {
            InputArgs {
                input_command: InputCommand::Error,
                arg_matches: ArgMatches::default(),
            }
        }
    }

    pub fn input_command(&self) -> InputCommand {
        self.input_command
    }

    pub fn get_matches(&self) -> &ArgMatches {
        &self.arg_matches
    }

    pub fn get_root_path(&self) -> PathBuf {
        match &self.arg_matches.value_of("PATH") {
            Some(path) => { Path::new(path).to_path_buf() }
            None => {
                current_dir().unwrap_or_else(|err| {
                    println!("{} {}", "Error accessing current_dir:".red(), err);
                    process::exit(1);
                })
            }
        }
    }
}