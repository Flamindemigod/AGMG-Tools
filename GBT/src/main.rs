mod modules;
pub mod utils;

use crate::modules::{config::does_config_exist, script::run_script};
use clap::{Args, Command, Error, FromArgMatches, Parser, Subcommand};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::*;
use modules::{
    archive::{archive_project, export_mod, run_unzip},
    config::Config,
    watcher::watch,
    *,
};
use std::{path::PathBuf, process::exit, sync::Mutex};
use utils::version::Version;

lazy_static! {
    pub static ref VERSION: Version = Version::from_str(env!("CARGO_PKG_VERSION")).unwrap();
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
    #[command(subcommand)]
    Run(RunBranch),
    Clean,
    /// Used to Build Textures
    // #[command(hide = true, hide_possible_values=true)]
    Build(BuildBranch),
    /// Used to Watch Updates To Files (For Script Use)
    #[command(hide = true, hide_possible_values = true)]
    Watch,
    /// Used to Generate Ini for Mod
    // #[command(hide = true, hide_possible_values=true)]
    GenIni,
    /// Used Link Mod to 3DMigoto
    // #[command(hide = true, hide_possible_values=true)]
    Link(LinkBranch),
    /// Exports Finished Mods as a Zip File
    Export(ExportBranch),
    /// Compresses Entire Project Directory to Folder.
    Archive(ArchiveBranch),
    /// Exports Finished Mods as a Zip File
    Extract(ExtractBranch),

    /// Check for Updates and Update if Available
    Update,
}

#[derive(Args, Debug, Clone)]
struct InitBranch {
    #[arg(default_value = ".")]
    /// Path to Where you want the project to Initialize
    project_path: PathBuf,
}

#[derive(Debug, Clone)]
struct RunBranch {
    script: String,
}

impl FromArgMatches for RunBranch {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        match matches.subcommand() {
            Some(script) => Ok(Self {
                script: script.0.to_string(),
            }),
            None => Err(Error::raw(
                clap::error::ErrorKind::MissingSubcommand,
                "Valid subcommands are `add` and `remove`",
            )),
        }
    }
    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        match matches.subcommand() {
            Some(script) => {
                *self = Self {
                    script: script.0.to_string(),
                }
            }
            None => (),
        }
        Ok(())
    }
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

impl Subcommand for RunBranch {
    fn augment_subcommands(cmd: clap::Command) -> clap::Command {
        let mut new_cmd = cmd;
        let binding = CONFIG.lock().unwrap().clone();
        let scripts = binding
            .scripts_parsed
            .keys()
            .sorted()
            .collect_vec()
            .to_owned();
        for script in scripts {
            let s_slice: &str = string_to_static_str(script.clone());
            new_cmd = new_cmd.subcommand(Command::new(s_slice));
        }
        new_cmd
    }
    fn augment_subcommands_for_update(cmd: clap::Command) -> clap::Command {
        let mut new_cmd = cmd;
        let binding = CONFIG.lock().unwrap().clone();
        let scripts = binding.scripts.keys().sorted().collect_vec().to_owned();
        for script in scripts {
            let s_slice: &str = string_to_static_str(script.clone());
            new_cmd = new_cmd.subcommand(Command::new(s_slice));
        }
        new_cmd
    }
    fn has_subcommand(name: &str) -> bool {
        let binding = CONFIG.lock().unwrap().clone();
        let scripts = binding.scripts_parsed.keys().collect_vec().to_owned();
        scripts.iter().any(|f| name == f.as_str())
    }
}

#[derive(Args, Debug, Clone)]
struct BuildBranch {
    /// Force Rebuild
    #[arg(short, default_value_t = false)]
    force: bool,
}

#[derive(Args, Debug, Clone)]
struct LinkBranch {
    /// Use Symlink
    #[arg(short, long, default_value_t = false)]
    symlink: bool,
}

#[derive(Args, Debug, Clone)]
struct ArchiveBranch {
    #[arg(default_value = "..")]
    /// Path to Where you want the project archive to be stored
    archive_path: PathBuf,
}

#[derive(Args, Debug, Clone)]
struct ExportBranch {
    #[arg(default_value = ".")]
    /// Path to Where you want the project archive to be stored
    export_path: PathBuf,
}

#[derive(Args, Debug, Clone)]
struct ExtractBranch {
    #[arg(required = true)]
    /// Path to the archived Project Zip
    path_to_project_zip: PathBuf,

    #[arg(default_value = ".")]
    /// Path to Where you want the Project to be extracted to
    extract_path: PathBuf,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct CLI {
    /// Main Command To Execute
    #[command(subcommand)]
    main_command: Branches,

    /// Debug mode (-d, -dd, -ddd, etc)
    #[arg(short, action=clap::ArgAction::Count, default_value="3")]
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
        Branches::Run(script) => run_script(script.script, cli.debug),
        Branches::Clean => trace!("In Clean"),
        Branches::Build(build) => build!(build.force),
        Branches::Watch => watch(),
        Branches::GenIni => gen_ini::gen_ini(),
        Branches::Link(link) => linker::link(link.symlink),
        Branches::Update => updater::update(),
        Branches::Archive(archive) => archive_project(&archive.archive_path),
        Branches::Extract(extract) => {
            run_unzip(&extract.path_to_project_zip, &extract.extract_path)
        }
        Branches::Export(export) => export_mod(&export.export_path),
    }
}
