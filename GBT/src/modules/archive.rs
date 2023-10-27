use anyhow::{Error, Result};
use log::{error, info, trace};
use normpath::PathExt;
use std::{
    fs::{self, File},
    io::{copy, Read, Write},
    path::{Path, PathBuf},
    process::exit,
};
use walkdir::WalkDir;
use zip::{
    read::ZipArchive,
    write::{FileOptions, ZipWriter},
    CompressionMethod,
};

use crate::CONFIG;

use super::config::does_config_exist;

fn zip_dir(source: &PathBuf, target: &Path) -> Result<()> {
    if !source.is_dir() {
        return Err(Error::msg("Source is not a Dir"));
    }
    let it = WalkDir::new(source).into_iter().filter_map(|e| e.ok());
    let file = File::create(target).unwrap();

    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(source).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            trace!("adding file {path:?} as {name:?} ...");
            zip.start_file(name.to_str().unwrap().to_owned(), options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            trace!("adding dir {path:?} as {name:?} ...");
            zip.add_directory(name.to_str().unwrap().to_owned(), options)?;
        }
    }
    zip.finish()?;
    Ok(())
}

fn unzip(zip_path: &PathBuf, target: &PathBuf) -> Result<()> {
    if !zip_path.is_file() {
        return Err(Error::msg("zipPath is not a File"));
    }
    let zip_file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => target.join(path),
            None => continue,
        };
        trace!("{:}", outpath.display());
        if (*file.name()).ends_with('/') {
            trace!("File {} extracted to \"{}\"", i, outpath.display());
            fs::create_dir_all(&outpath).unwrap();
        } else {
            trace!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }

    Ok(())
}

pub fn run_unzip(source: &PathBuf, target: &PathBuf) {
    info!(
        "Extracting Archive from {:} to {:}",
        source.display(),
        target.display()
    );
    unzip(source, target).expect("Failed to Extract")
}

pub fn run_zip(source: &PathBuf, target: &PathBuf) {
    zip_dir(&source.normalize().unwrap().into_path_buf(), target).expect("Failed to Compress")
}

pub fn export_mod(target: &PathBuf) {
    if !does_config_exist() {
        error!("Config does not exist. Exiting....");
        exit(1);
    }
    let project_path = &target
        .normalize()
        .unwrap()
        .into_path_buf()
        .join(format!("{}.zip", CONFIG.lock().unwrap().project_name));
    info!("Exporting Mod to {:}", project_path.display());
    run_zip(&PathBuf::from("./Output"), project_path);
    info!("Mod Exported");
}

pub fn archive_project(target: &PathBuf) {
    if !does_config_exist() {
        error!("Config does not exist. Exiting....");
        exit(1);
    }
    let project_path = &target
        .normalize()
        .unwrap()
        .into_path_buf()
        .join(format!("{}.zip", CONFIG.lock().unwrap().project_name));
    info!("Compressing Project Archive to {:}", project_path.display());
    run_zip(&PathBuf::from("."), project_path);
    info!("Finished Compressing Project Archive. Exiting...");
}
