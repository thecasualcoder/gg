use std::env::ArgsOs;

use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputCommand {
    Status,
    Create,
    Error,
}

pub struct InputArgs<'a> {
    pub input_command: InputCommand,
    pub arg_matches:  ArgMatches<'a>,
    pub error_input: bool,
}

impl<'a> InputArgs<'a> {
    pub fn parse_inputs(args: ArgMatches) -> InputArgs {
        match args.subcommand_matches("status") {
            Some(matches) => InputArgs {
                input_command: InputCommand::Status,
                arg_matches: matches.to_owned(),
                error_input: false,
            },
            _ => InputArgs {
                input_command: InputCommand::Error,
                arg_matches: ArgMatches::default(),
                error_input: true,
            }
        }
    }

    pub fn input_command(&self) -> InputCommand {
        self.input_command
    }

    pub fn get_matches(&self) -> &ArgMatches {
        &self.arg_matches
    }
}