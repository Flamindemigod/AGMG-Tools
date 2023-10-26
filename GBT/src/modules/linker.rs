use std::{process::exit, path::PathBuf};
use normpath::PathExt;
use log::{error, warn};

use crate::{utils::copy::copy, CONFIG};

use super::config::does_config_exist;

pub fn link(should_symlink: bool) {
    if !does_config_exist() {
        error!("Project Config Does Not Exist. Exiting..");
        exit(1);
    }
    let migoto_path = CONFIG.lock().unwrap().migoto_path.to_owned();
    if migoto_path.is_none() {
        warn!("Migoto Path Not Exist. Exiting..");
        exit(0);
    }
    let project_name = CONFIG.lock().unwrap().project_name.to_owned();
    copy(
        &PathBuf::from("./Output").normalize().unwrap().into(),
        &migoto_path.unwrap().join("Mods").join(project_name),
        should_symlink,
    ).expect("Copy Failed");
}
