#### Git Governance
[![Build Status](https://travis-ci.org/thecasualcoder/gg.svg?branch=master)](https://travis-ci.org/thecasualcoder/gg)

A tool to manage multiple git repositories. 
This does not aim to replace git in any way but tries to reduce the work in managing multiple git repositories at once.


##### Installation

Using Brew:

```bash
brew tap thecasualcoder/stable
brew install gg
```

Installing from source:
```bash
git clone https://github.com/thecasualcoder/gg.git
cd gg
cargo install --path .
```

> Note: Recommaded rustc/cargo version: 1.36.0 and above or 1.37.0-nightly and above 


##### Usage:

Help:
```bash
$ gg --help
```
![Help](/gifs/ggHelp.gif)

Status:
Shows status of all git repos from current directory. Traverses inside directories also. 
To traverse through hidden directories use the `-i` flag. By default hidden directories will not be traversed.

```bash
$ gg status
```
![Status](/gifs/ggStatus.gif)

Create:
Creates a remote repository and clones it to the local path specified. Remote repository is created based on the GITHUB_TOKEN provided.
GITHUB_TOKEN can be passed as an env variable or can be given as an argument through the `-t` flag
```bash
$ gg create -r <repo_name> -l <local_path>
```
![Create](/gifs/ggCreate.gif)

Fetch:
Fetches from all git repositories starting from current directory. Traverses inside directories also.
Currently Fetch only uses the private key `id_rsa` to authenticate and will fail to fetch if it is not enough. Failure to fetch one repo will not fail others
To traverse through hidden directories use the `-i` flag. By default hidden directories will not be traversed.
```bash
$ gg fetch 
```
![Fetch](/gifs/ggFetch.gif)

Clone:
Clones repositories based on the flags passed and the configuration given in the `.ggConf.yaml` file.
```bash
$ gg clone -r <remote_url_1> -r <remote_url_2> -l <local_path>  
```
![Clone](/gifs/ggClone.gif)

Config file:

The config file can be specified via the `-c` flag. By default it tries to find `.ggConf.yaml`.
Example config file:

```yaml
skipDirectories:
  - ignore
cloneRepos:
  - remoteURL: https://github.com/golang/net.git
    localPath: here/net
  - remoteURL: https://github.com/golang/net.git
    localPath: there/net
```

##### Contributing:
[Please refer the github projects board](https://github.com/thecasualcoder/gg/projects/1)
If you want some feature, please create an issue and if possible feel free to raise PR too.