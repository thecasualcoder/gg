use std::collections::HashMap;
use std::error::Error;
use std::{env, process};

use clap::{App, Arg, SubCommand};
use colored::*;
use reqwest::{Client, RequestBuilder};

use crate::clone::GitClone;
use crate::input_args::InputArgs;
use crate::progress::ProgressTracker;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("create")
        .arg(
            Arg::with_name("repo_path")
                .short("r")
                .required(true)
                .takes_value(true)
                .help("path at which to create the local repo. Defaults to '.'"),
        )
        .arg(
            Arg::with_name("platform")
                .short("p")
                .default_value("github")
                .takes_value(true)
                .help("the remote platform for the git repo. Defaults to github"),
        )
        .arg(
            Arg::with_name("token")
                .short("t")
                .help("the access token to create the repo remotely"),
        )
}

pub fn create(args: InputArgs) {
    let matches = args.get_matches();
    let root_path = args.get_root_path("repo_path");
    let repo_name = root_path
        .to_str()
        .expect(format!("{}", "Error in converting directory to string".red()).as_str());

    let mut token = String::from(matches.value_of("token").unwrap_or(""));
    if token == "" {
        if env::var("GITHUB_TOKEN").is_err() {
            println!(
                "{}",
                "GITHUB_TOKEN is missing. Set this as a flag using -t or as an env variable".red()
            );
            process::exit(1)
        } else {
            token = env::var("GITHUB_TOKEN").unwrap()
        }
    }

    let platform = matches.value_of("platform").unwrap();

    let remote_url = create_remote(token, platform, repo_name).unwrap_or_else(|err| {
        println!("{} {}", "Failed creating a remote repo:".red(), err);
        process::exit(1);
    });

    let clone = GitClone {
        remote_url,
        local_path: root_path,
        use_ssh: false,
    };

    let multi_bars = ProgressTracker::new(matches.value_of("jobs").and_then(|e| e.parse().ok()));
    multi_bars.start_task(clone);
    multi_bars.join().unwrap();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GitPlatform {
    Github,
}

impl GitPlatform {
    fn from_str(platform: &str) -> GitPlatform {
        match platform {
            "github" => GitPlatform::Github,
            _ => panic!("{}", "Unsupported platform".red()),
        }
    }

    fn create_api(&self, client: Client, repo_name: String, token: String) -> RequestBuilder {
        match *self {
            GitPlatform::Github => {
                let mut data = HashMap::new();
                data.insert("name", repo_name);
                client
                    .post("https:/api.github.com/user/repos")
                    .header("Authorization", format!("token {}", token))
                    .header("Accept", "application/vnd.github.v3+json")
                    .header("Content-Type", "application/json")
                    .json(&data)
            }
        }
    }
}

struct GitRemoteRepo {
    platform: GitPlatform,
    token: String,
    repo_name: String,
}

impl GitRemoteRepo {
    fn create(self) -> Result<String, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let request_builder = self.platform.create_api(client, self.repo_name, self.token);
        let mut response = request_builder.send()?;
        let returned_json: serde_json::Value = response.json()?;
        let errors = returned_json["errors"]
            .as_array()
            .unwrap_or(&Vec::new())
            .to_owned();
        if errors.len() > 0 {
            let message = errors[0]
                .get("message")
                .expect("Failed to get error message")
                .to_owned();
            println!("{} {:#?}", "Error message:".red(), message.to_string());
            process::exit(1)
        }
        let url = returned_json["clone_url"]
            .as_str()
            .expect("Failed to get remote url from response");
        Ok(url.to_owned())
    }
}

fn create_remote(token: String, platform: &str, repo_name: &str) -> Result<String, Box<dyn Error>> {
    let remote_repo = GitRemoteRepo {
        platform: GitPlatform::from_str(platform),
        token,
        repo_name: String::from(repo_name),
    };
    remote_repo.create()
}
