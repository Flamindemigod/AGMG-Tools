mod modules;
pub mod utils;

use std::{path::PathBuf, sync::{Arc, Mutex}};
use clap::{Parser, Subcommand, Args};
use lazy_static::lazy_static;
use log::*;
use modules::{*, config::Config};

use crate::modules::{git::Git, config::does_config_exist};

lazy_static!(
    pub static ref CONFIG:Mutex<Config> = Mutex::new(config::Config::default());
);


#[derive(Subcommand, Debug, Clone)]
enum Branches {
    Init(InitBranch),
    Run, 
    Test,
    Clean,
} 



#[derive(Args, Debug, Clone)]
struct InitBranch {
    #[arg(default_value = ".")]
    /// Path to Where you want the project to Initialize
    project_path: PathBuf
}

#[derive(Args, Debug, Clone)]
struct RunBranch {
    // #[arg(try_get_matches_from = ["resx"])]
    /// Path to Where you want the project to Initialize
    script: String
}


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CLI{
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
    let mut std_err = stderrlog::new();
    std_err.module(module_path!()).verbosity(usize::from(cli.debug));

    #[cfg(feature = "timestamps")]
    std_err.timestamp(cli.ts.unwrap_or(stderrlog::Timestamp::Off));

    std_err.init().unwrap();

    info!("{cli:#?}");
    if does_config_exist(){
        CONFIG.lock().unwrap().load_project_conf();
    }

    match cli.main_command {
        Branches::Init(init) => scaffold::scaffold(init.project_path),
        Branches::Run => trace!("In Run"),
        Branches::Test => trace!("In Test"),
        Branches::Clean => trace!("In Clean"),
        _ => unreachable!("How'd You get here")
    }

    // let url = "https://github.com/SilentNightSound/GI-Model-Importer-Assets/tree/main/SkillData/FoxPole";
    // let path = PathBuf::from(url);
    // let git = Git::default().load(&path).unwrap();

    // match git.download(PathBuf::from("./Testing")){
    //     Ok(_) => todo!(""),
    //     Err(e) => error!("{:#?}", e),
    // }

}
