use std::fs::File;
use std::io::Write;

use clap::{App, Arg, SubCommand};
use colored::*;
use git2::{Repository};
use regex::Regex;

use crate::clone::GitRepo;
use crate::conf::{GGConf, SSHConfig};
use crate::dir::DirectoryTreeOptions;
use crate::input_args::InputArgs;
use crate::SSH_CONF;

pub fn sub_command<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("config")
        .arg(
            Arg::with_name("root_path")
                .short("r")
                .default_value(".")
                .takes_value(true)
                .help("path of the repo where branches are to be compared. Defaults to '.'"),
        ).arg(
        Arg::with_name("traverse-hidden")
            .short("i")
            .help("traverse through hidden directories also"),
    )
}

pub fn config(args: InputArgs, filter_list_regex: Vec<Regex>, filter_list: Vec<String>,
              mut existing_clone_repos: Vec<GitRepo>) {
    let root_path = args.get_root_path("PATH");
    let root = root_path
        .to_str()
        .expect(format!("{}", "Error in converting directory to string".red()).as_str());
    let matches = args.get_matches();
    let filter_hidden = matches.is_present("traverse-hidden");

    let dir_tree_with_options = DirectoryTreeOptions {
        filter_list: filter_list_regex.clone(),
        filter_hidden,
    };

    let mut git_repos: Vec<GitRepo> = Vec::new();
    dir_tree_with_options
        .process_directories(root)
        .flat_map(|dir| {
            dir.ok().and_then(|d| {
                if d.file_name().eq(".git") {
                    d.path().parent().map(|e| e.to_path_buf())
                } else {
                    None
                }
            })
        })
        .map(|dir| {
            let local_path = dir.clone().to_str().expect("Failed to extract string from path").to_string();
            let repo = Repository::open(dir).expect("Failed to open git repo");
            let remote = repo.find_remote("origin").expect("Failed to get remote with name origin in the repo");
            let remote_url = remote.url();

            GitRepo {
                remote_url: remote_url.expect("Failed to get remote url as string").to_string(),
                local_path: local_path,
            }
        }).for_each(|repo| git_repos.push(repo));

    git_repos.append(&mut existing_clone_repos);

    let ssh_conf = SSH_CONF.lock().unwrap();

    let config = SSHConfig {
        private_key: ssh_conf.private_key.clone(),
        username: ssh_conf.username.clone(),
        ssh_agent: ssh_conf.ssh_agent,
    };

    let new_conf = GGConf {
        filter_list: filter_list,
        filter_list_regex: filter_list_regex,
        clone_repos: git_repos,
        ssh_config: Some(config),
    };

    let yaml_string = serde_yaml::to_string(&new_conf).expect("Failed to parse yaml string from conf object");
    println!("{}", yaml_string);
    let new_path = format!("{}/.ggConf.new.yaml", root);
    let mut file = File::create(new_path.clone()).expect("Failed to create new conf file");
    file.write_all(yaml_string.as_bytes()).expect("failed to write ggConf content to new conf file");
    println!("{} {}", "Yaml saved at:".green(), new_path.blue())
}