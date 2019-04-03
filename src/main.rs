use git2::Repository;
use log::{info, trace, warn};
use serde_derive::{Deserialize, Serialize};
use simplelog::*;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
  /// The configuration file to use, defaults to $HOME/.config/workspace.toml
  /// Can be overridden with WORKSPACE_CONFIG environment variable
  #[structopt(long = "config")]
  config_path: Option<PathBuf>, 
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkspaceConfig {
  projects: HashMap<String, PathBuf>
}

// TODO: Fix with proper error propagation and logging
fn print_and_quit <E: std::error::Error, V> (result: Result<V, E>) -> V {
  match result {
    Ok(value) => value,
    Err(e) => {
      println!("Fatal Error:");
      println!("{:?}", e);
      std::process::exit(1);
    }
  }
}

fn handle_project(name: &str, path: &Path) {
  println!("\tLooking at {} in {}...", name, path.display());

  let repo = match Repository::init(path) {
    Ok(repo) => repo,
    // TODO: cleanup with logging
    Err(e) => panic!("Failed to init repo for {}: {}", path.display(), e),
  };

  let diff = match repo.diff_index_to_workdir(None, None) {
    Ok(diff) => diff,
    // TODO: cleanup with logging
    Err(e) => panic!("Failed to get diff for {}: {}", path.display(), e),
  };

  let stats = match diff.stats() {
    Ok(stats)  => stats,
    // TODO: cleanup with logging
    Err(e) => panic!("Failed to get diff stats for {}: {}", path.display(), e),
  };

  let files_changed = stats.files_changed();
  if files_changed > 0 {
    println!("\t\t{} has {} changed files, commiting!", name, files_changed);
  } else {
    println!("\t\t{} has no changed filed, skipping", name);
  }
}

fn main() {
  let opt = Opt::from_args();

  // The config path is chosen in this order:
  //  1. Check for environment variable "WORKSPACE_CONFIG"
  //  2. If not that then see if it was passed via command line
  //  3. If not that then try to find user's home directory and build default path
  //  If that all fails then quit
  let config_path = match (env::var("WORKSPACE_CONFIG"), opt.config_path, env::var("HOME")) {
    (Ok(env_path), _, _) => PathBuf::from(env_path),
    (_, Some(cli_path), _) => cli_path,
    (_, _, Ok(home_path)) => {
      let mut default_path = PathBuf::from(home_path);
      default_path.push(".config");
      default_path.push("workspace.toml");
      default_path
    }
    _ => {
      // TODO: fix with logging
      panic!("No config file passed via WORKSPACE_CONFIG or --config, and HOME not defined");
    }
  };

  // Open and deserialize config
  let mut config_file = print_and_quit(File::open(&config_path));
  let mut config_contents = String::new();
  print_and_quit(config_file.read_to_string(&mut config_contents));
  let config: WorkspaceConfig = print_and_quit(toml::from_str(&config_contents));
  println!("Using Config: {:?}", config);

  for (name, path) in &config.projects {
    handle_project(name, path);
  }
}
