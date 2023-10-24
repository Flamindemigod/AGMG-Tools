mod modules;
pub mod utils;

use clap::{Args, Parser, Subcommand};
use lazy_static::lazy_static;
use log::*;
use modules::{config::Config, *, watcher::watch};
use utils::test::test;
use std::{path::PathBuf, process::exit, sync::Mutex};

use crate::modules::config::does_config_exist;

lazy_static! {
    pub static ref CONFIG: Mutex<Config> = {
        let mut config = config::Config::default();
        if does_config_exist() {
            config.load_project_conf();
        }
        Mutex::new(config)
    };
}

#[derive(Subcommand, Debug, Clone)]
enum Branches {
    Init(InitBranch),
    Run,
    Clean,
    Test,
    #[command(hide = true)]
    Build(BuildBranch),
    #[command(hide = true)]
    Watch,
}

#[derive(Args, Debug, Clone)]
struct InitBranch {
    #[arg(default_value = ".")]
    /// Path to Where you want the project to Initialize
    project_path: PathBuf,
}

#[derive(Args, Debug, Clone)]
struct RunBranch {
    /// Path to Where you want the project to Initialize
    script: String,
}

#[derive(Args, Debug, Clone)]
struct BuildBranch {
    /// Force Rebuild
    #[arg(short, default_value_t = false)]
    force: bool,
}



#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CLI {
    /// Main Command To Execute
    #[command(subcommand)]
    main_command: Branches,

    /// Debug mode (-d, -dd, -ddd, etc)
    #[arg(short, action=clap::ArgAction::Count)]
    debug: u8,

    #[cfg(feature = "timestamps")]
    /// Timestamp (sec, ms, ns, none)
    #[arg(short, long = "timestamp")]
    ts: Option<stderrlog::Timestamp>,
}

fn main() {
    let cli = CLI::parse();
    if !CONFIG.lock().unwrap().valid_exe() {
        error!("Exe Validation Failed. Exiting");
        exit(1);
    }
    let mut std_err = stderrlog::new();
    std_err
        .module(module_path!())
        .verbosity(usize::from(cli.debug));

    #[cfg(feature = "timestamps")]
    std_err.timestamp(cli.ts.unwrap_or(stderrlog::Timestamp::Off));

    std_err.init().unwrap();
    

    match cli.main_command {
        Branches::Init(init) => scaffold::scaffold(init.project_path),
        Branches::Run => {
            use std::{thread, time};
            let mut i = 0;
            print!("{:}", i);
            loop {
                i+=1;
                println!("{:}", i);
                thread::sleep(time::Duration::from_secs(2));
                if i == 20{
                    break;
                }
        }},
        Branches::Test => test(),
        Branches::Clean => trace!("In Clean"),
        Branches::Build(build) => build!(build.force),
        Branches::Watch => watch(),
    }
}
