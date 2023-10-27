use crate::CONFIG;

use super::config::does_config_exist;
use file_diff::diff;
use log::{error, info, trace};
use notify_debouncer_full::{new_debouncer, notify::*};
use rayon::prelude::*;
use serde_json;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::exit,
    time::Duration,
};

fn paths(files: &mut Vec<String>, paths: &Vec<PathBuf>) -> Option<PathBuf> {
    for path in paths {
        let is_match = files.par_iter().any(|file| {
            if diff(path.to_str().unwrap(), file.as_str()) {
                return true;
            }
            return false;
        });
        if is_match {
            if diff(path.to_str().unwrap(), "./Config.yml") {
                CONFIG.lock().unwrap().load_project_conf();
                build_watch_files(files);
            }
            trace!("matched: {:#?}", path);
            return Some(path.to_path_buf());
        } else {
            if let Some(extension) = path.extension() {
                if extension.to_str().unwrap() == "ini" {
                    build_watch_files(files);
                }
            }
        }
    }
    return None;
}

fn build_watch_files(files: &mut Vec<String>) {
    info!("Building Watch Files");
    *files = vec![];
    files.push("./Config.yml".to_owned());
    let ini_res = glob::glob("./Output/*.ini").unwrap().find(|_| true);
    if let Some(ini_path) = ini_res {
        match ini_path {
            Ok(ini_path) => files.push(ini_path.to_str().unwrap().to_owned()),
            Err(_) => (),
        }
    }

    let mut tex = CONFIG
        .lock()
        .unwrap()
        .model_textures
        .values()
        .map(|f| f.paths.to_vec())
        .flatten()
        .map(|path| path.to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    files.append(&mut tex);

    tex = CONFIG
        .lock()
        .unwrap()
        .texture_textures
        .values()
        .map(|f| f.paths.to_vec())
        .flatten()
        .map(|path| path.to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    files.append(&mut tex);
    info!("Watch Files Built");
}

pub fn watch() {
    if !does_config_exist() {
        error!("Config does not exist in current directory. Exiting");
        exit(1);
    }
    info!("Initializing Watcher");
    let mut watched_files = vec![];
    build_watch_files(&mut watched_files);

    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx).unwrap();

    debouncer
        .watcher()
        .watch(Path::new("."), RecursiveMode::Recursive)
        .unwrap();
    debouncer
        .watcher()
        .configure(Config::default().with_compare_contents(true))
        .unwrap();
    debouncer
        .cache()
        .add_root(Path::new("."), RecursiveMode::Recursive);

    for result in rx {
        match result {
            Ok(events) => events
                .iter()
                .map(|event| paths(&mut watched_files, &event.paths))
                .filter(|f| f.is_some())
                .map(|f| f.unwrap())
                .collect::<HashSet<_>>()
                .par_iter()
                .for_each(|path| println!("{:}", serde_json::to_string(path).unwrap())),
            Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
        }
    }
}
