use crate::CONFIG;

use super::config::does_config_exist;
use file_diff::diff;
use log::{error, trace};
use notify_debouncer_full::{new_debouncer, notify::*};
use rayon::prelude::*;
use serde::Deserialize;
use serde_json;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::exit,
    time::Duration,
};


fn paths(files: &mut Vec<String>, paths: &Vec<PathBuf>) -> Option<PathBuf> {
    for path in paths {
        trace!("Checking Path: {:#?}", path);
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
            return Some(path.to_path_buf());
        }
    }
    return None;
}

fn build_watch_files(files: &mut Vec<String>) {
    *files = vec![];
    files.push("./Config.yml".to_owned());
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
}

pub fn watch() {
    if !does_config_exist() {
        error!("Config does not exist in current directory. Exiting");
        exit(1);
    }

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
                .map(|event| {
                    trace!("{event:#?}");
                    paths(&mut watched_files, &event.paths)
                })
                .filter(|f| f.is_some())
                .map(|f| f.unwrap())
                .collect::<HashSet<_>>()
                .par_iter()
                .for_each(|path| println!("{:}", serde_json::to_string(path).unwrap())),
            Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
        }
    }
}
