mod modules;
pub mod utils;

use std::path::PathBuf;
use clap::{Parser, Subcommand, Args};
use indicatif::MultiProgress;
use log::*;
use modules:: *;

use crate::modules::git::{Git, RepoItem};


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

    warn!("{cli:#?}");

    match cli.main_command {
        Branches::Init(init) => trace!("In Init {:#?}", init.project_path),
        Branches::Run => trace!("In Run"),
        Branches::Test => trace!("In Test"),
        Branches::Clean => trace!("In Clean"),
        _ => unreachable!("How'd You get here")
    }

    let url = "https://github.com/SilentNightSound/GI-Model-Importer-Assets/tree/main/PlayerCharacterData/Furina";
    let path = PathBuf::from(url);
    let git = Git::default().load(&path).unwrap();

    match git.download(PathBuf::from("./Testing")){
        Ok(_) => todo!(""),
        Err(e) => error!("{:#?}", e),
    }

}
