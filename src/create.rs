use clap::{App, Arg, SubCommand, ArgMatches};
use std::env::current_dir;
use std::path::Path;
use std::{fs, env};
use crate::create::GitPlatform::Github;
use reqwest::{RequestBuilder, Client};
use std::collections::HashMap;
use std::error::Error;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("create")
        .arg(Arg::with_name("repo_path")
            .short("r")
            .required(true)
            .takes_value(true)
            .help("path at which to create the local repo. Defaults to '.'"))
        .arg(Arg::with_name("platform")
            .short("p")
            .default_value("github")
            .takes_value(true)
            .help("the remote platform for the git repo. Defaults to github"))
        .arg(Arg::with_name("token")
            .short("t")
            .help("the access token to create the repo remotely")
        )
}

pub fn create(args: &ArgMatches) {
    let current_dir = current_dir()
        .expect("Failed to get current directory");

    let current_dir_path = current_dir
        .to_str()
        .expect("failed to convert current directory path to string");

    let path = args.value_of("repo_path")
        .unwrap_or(current_dir_path);

    let repo_name = Path::new(path)
        .file_name()
        .expect("failed to get repo_name")
        .to_str()
        .expect("failed to convert path to string");

    let path = args.value_of("repo_path")
        .unwrap_or(current_dir_path);

    let mut api_token = String::new();
    let token = args.value_of("token").unwrap_or("");
    if token == "" {
        if env::var("GITHUB_TOKEN").is_err() {
            panic!("GITHUB_TOKEN is missing. Set this as a flag or as an env variable")
        } else {
            api_token = env::var("GITHUB_TOKEN").unwrap()
        }
    } else {
        api_token = String::from(token);
    }

    let platform = args.value_of("platform").unwrap();

    create_remote(api_token, platform, repo_name);

//    match create_local_directory(path) {
//        Ok(_) => {
//            create_remote(api_token, platform, repo_name)
//        }
//        Err(e) => {
//            panic!("error while creating local directory")
//        }
//    }
}

fn create_local_directory(path: &str) -> std::io::Result<()> {
    fs::create_dir(path)?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GitPlatform {
    Github
}

impl GitPlatform {
    fn as_str(&self) -> &'static str {
        match *self {
            GitPlatform::Github => "github"
        }
    }

    fn from_str(platform: &str) -> GitPlatform {
        match platform {
            "github" => GitPlatform::Github,
            _ => {
                panic!("Unsupported platform")
            }
        }
    }

    fn create_api(&self, client: Client, repo_name: String, token: String) -> RequestBuilder {
        match *self {
            GitPlatform::Github => {
                let mut data = HashMap::new();
                data.insert("name", repo_name);
                client.post("https:/api.github.com/user/repos")
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
    fn create(self) -> Result<(), Box<dyn Error>> {
        let client = reqwest::Client::new();
        let mut request_builder = self.platform.create_api(client, self.repo_name, self.token);
        let response_status = request_builder.send()?
            .status();

        if response_status.as_str() != "201" {
            panic!("Creating remote repo failed");
        }
        Ok(())
    }
}

fn create_remote(token: String, platform: &str, repo_name: &str) -> Result<(), Box<dyn Error>> {
    let remote_repo = GitRemoteRepo {
        platform: GitPlatform::from_str(platform),
        token: token,
        repo_name: String::from(repo_name),
    };
    remote_repo.create()?;
    Ok(())
}