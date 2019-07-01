use clap::ArgMatches;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputCommand {
    Status,
    Create,
    Error,
}

impl InputCommand {
    fn as_str(&self) -> &'static str {
        match *self {
            InputCommand::Status => "status",
            InputCommand::Create => "create",
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
}