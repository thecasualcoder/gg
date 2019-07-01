use clap::{App, Arg, SubCommand, ArgMatches};

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("create")
        .arg(Arg::with_name("repo_path")
            .short("r")
            .default_value(".")
            .takes_value(true)
            .help("path at which to create the local repo. Defaults to '.'"))
        .arg(Arg::with_name("repo_name")
            .short("n")
            .takes_value(true)
            .help("name of the repo"))
        .arg(Arg::with_name("platform")
            .short("p")
            .default_value("github")
            .takes_value(true)
            .help("the remote platform for the git repo. Defaults to github")
        )
        .arg(Arg::with_name("token")
            .short("t")
            .takes_value(true)
            .required(true)
            .help("the access token to create the repo remotely")
        )
}

pub fn create(_args: &ArgMatches) {}