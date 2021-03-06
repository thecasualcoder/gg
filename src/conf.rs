use std::error::Error;
use std::fs;
use std::path::Path;

use colored::Colorize;
use git2::{Cred, CredentialType, Error as GitError};
use regex::Regex;
use crate::SSH_CONF;
use serde::{Deserialize, Serialize};

use crate::clone::GitRepo;

// Todo: This will never be serialized. Try removing Serialize.
#[derive(Debug, Serialize, Deserialize)]
pub struct GGConf {
    #[serde(alias = "skipDirectories")]
    #[serde(rename = "skipDirectories")]
    #[serde(default)]
    pub filter_list: Vec<String>,

    #[serde(skip)]
    pub filter_list_regex: Vec<Regex>,

    #[serde(alias = "cloneRepos")]
    #[serde(rename = "cloneRepos")]
    #[serde(default)]
    // Todo: Add validations on this field. It should not allow empty key/values.
    pub clone_repos: Vec<GitRepo>,

    #[serde(alias = "ssh")]
    #[serde(rename = "ssh")]
    #[serde(default)]
    pub ssh_config: Option<SSHConfig>,
}

impl GGConf {
    pub fn default() -> GGConf {
        GGConf {
            filter_list: vec![],
            filter_list_regex: vec![],
            clone_repos: vec![],
            ssh_config: None,
        }
    }
}

pub fn read_conf_file(conf_file: &str) -> Result<GGConf, Box<dyn Error>> {
    if Path::new(conf_file).exists() {
        let file = fs::File::open(conf_file)?;
        let mut config: GGConf = serde_yaml::from_reader(file)?;
        update_conf_file(&mut config)?;
        return Ok(config);
    }
    let mut default = GGConf::default();
    update_conf_file(&mut default)?;
    Ok(default)
}

fn update_conf_file<'a>(conf: &mut GGConf) -> Result<(), Box<dyn Error>> {
    create_filter_list(conf)?;
    Ok(())
}

fn create_filter_list<'a>(conf: &mut GGConf) -> Result<(), Box<dyn Error>> {
    let mut filter_list = Vec::new();
    let mut filters = conf.filter_list.clone();
    let defaults: Vec<String> = [".idea", ".DS_Store"].iter().map(|&s| s.into()).collect();
    defaults.iter().for_each(|def| {
        filters.push(def.to_owned());
    });

    filters.iter().for_each(|ignore| {
        let re =
            Regex::new(format!(r".*/{}?*", ignore).as_str()).expect("failed to construct regex");
        filter_list.push(re);
    });

    conf.filter_list = filters;
    conf.filter_list_regex = filter_list;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SSHConfig {
    #[serde(alias = "privateKey")]
    #[serde(rename = "privateKey")]
    pub private_key: String,

    pub username: String,

    #[serde(alias = "sshAgent")]
    #[serde(rename = "sshAgent")]
    #[serde(default)]
    pub ssh_agent: bool,
}

pub fn ssh_auth_callback(_user: &str, _user_from_url: Option<&str>, _cred: CredentialType) -> Result<Cred, GitError> {
    let ssh_conf = SSH_CONF.lock().unwrap();

    if ssh_conf.private_key.is_empty() {
        println!("{}", "Please set the private key to be used to authenticate".red());
    }

    if ssh_conf.username.is_empty() {
        println!("{}", "Please set the username to be used to authenticate".red());
    }

    if ssh_conf.ssh_agent {
        return Cred::ssh_key_from_agent(ssh_conf.username.as_str());
    }

    Cred::ssh_key(ssh_conf.username.as_str(),
                  None,
                  Path::new(&ssh_conf.private_key),
                  None)
}

