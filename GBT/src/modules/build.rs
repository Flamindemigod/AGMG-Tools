use crate::CONFIG;
use anyhow::Result;
use filetime::FileTime;
use log::{error, info, trace};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    env::current_dir,
    fs::{self, metadata},
    path::PathBuf,
    process::exit,
    sync::Arc,
};

use super::{config::TexUnit, dds::build_from_tex_unit};

fn build_texture_units(force: bool, textures: HashMap<String, TexUnit>) -> Result<Vec<()>> {
    textures
        .par_iter()
        .map(|(filename, texunit)| {
            let output_path = current_dir().unwrap().join("Output");
            let output_file_name = output_path.join(format!("{:}.dds", filename));
            fs::create_dir_all(&output_path)?;
            //Check for updates or force
            if force || needs_rebuild(texunit.paths.clone(), output_file_name.clone()) {
                build_from_tex_unit(texunit.clone(), output_file_name)
            } else {
                trace!("{:} does not need rebuild", filename);
                Ok(())
            }
        })
        .collect::<Result<Vec<_>, _>>()
}

pub fn build(force: bool) {
    let textures = CONFIG.lock().unwrap().model_textures.clone();
    let res = build_texture_units(force, textures);
    if res.is_err() {
        error!("Failed to Compile Model Texture Units");
        exit(1);
    }
    let textures = CONFIG.lock().unwrap().texture_textures.clone();
    let res = build_texture_units(force, textures);
    if res.is_err() {
        error!("Failed to Compile Texture Mod Texture Units");
        exit(1);
    }
    info!("Rebuild Complete");
}

fn needs_rebuild(source: Arc<[PathBuf]>, target: PathBuf) -> bool {
    if !target.exists() {
        return true;
    }
    let tmeta = metadata(target).expect("Failed to get File Metadata");
    let tfile_mtime = FileTime::from_last_modification_time(&tmeta);
    let res: Result<Vec<_>, _> = source
        .par_iter()
        .map(|f| {
            let imeta = metadata(f).expect("Failed to get File Metadata");
            let ifile_mtime = FileTime::from_last_modification_time(&imeta);
            if tfile_mtime.gt(&ifile_mtime) {
                Ok(())
            } else {
                Err(())
            }
        })
        .collect();
    if res.is_err() {
        return true;
    }
    return false;
}

#[macro_export]
macro_rules! build {
    () => {
        crate::modules::build::build(false)
    };
    ($a: expr) => {
        crate::modules::build::build($a)
    };
}
