#### Git Governance
[![Build Status](https://travis-ci.org/thecasualcoder/gg.svg?branch=master)](https://travis-ci.org/thecasualcoder/gg)

A tool to manage multiple git repositories


##### Installation

Using Brew:

```bash
brew tap thecasualcoder/stable
brew install gg
```

Using cargo:
```bash
git clone https://github.com/thecasualcoder/gg.git
cd gg
cargo install --path .
```

> Note: Recommaded rustc/cargo version: 1.36.0 and above or 1.37.0-nightly and above 


##### Usage:

Status:

```bash
$ pwd
$BASE_DIR/thecasualcoder

$ gg status
"$BASE_DIR/thecasualcoder/kube-fzf": new files
"$BASE_DIR/thecasualcoder/temp/gg": no changes
"$BASE_DIR/thecasualcoder/gg": modifications
"$BASE_DIR/thecasualcoder/homebrew-core": no changes
"$BASE_DIR/thecasualcoder/utils": modifications
"$BASE_DIR/thecasualcoder/pg-ping": no changes
"$BASE_DIR/thecasualcoder/homebrew-stable": modifications
"$BASE_DIR/thecasualcoder/file-utils": no changes
"$BASE_DIR/thecasualcoder/tztail": no changes
```

Create:
```bash
$ gg create
```

Config file:

The config file can be specified via the `-c` flag. By default it tries to find `.ggConf.yaml`(Will currently fail if this is empty or missing). 
Example config file:

```yaml
skipDirectories:
  - .idea
  - ignore
```

Currently only skip directories is in the config file. More features/bugs will be added/fixed around it.

##### TODO:
[Please refer the github projects board](https://github.com/thecasualcoder/gg/projects/1)

If you want some feature, please create an issue and if possible feel free to raise PR too.