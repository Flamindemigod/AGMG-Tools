use anyhow::Result;
use fs_extra::{
    dir::{copy as cp_dir, CopyOptions as cpo_dir},
    file::{copy as cp_file, CopyOptions as cpo_files},
};
use std::{fs, path::PathBuf};

#[cfg(target_family = "windows")]
fn symlink(source: &PathBuf, target: &PathBuf) -> Result<()> {
    use std::os::windows::fs::{symlink_dir, symlink_file};
    if source.is_dir() {
        symlink_dir(source, target)?;
    } else {
        symlink_file(source, target)?;
    }
    Ok(())
}

#[cfg(target_family = "unix")]
fn symlink(source: &PathBuf, target: &PathBuf) -> Result<()> {
    std::os::unix::fs::symlink(source, target)?;
    Ok(())
}

fn copy(source: &PathBuf, target: &PathBuf, should_symlink: bool) -> Result<()> {
    if should_symlink {
        symlink(source, target)?;
    }
    else {
        fs::create_dir_all(target)?;
        if source.is_dir() {
            let mut options = cpo_dir::new();
            options.content_only = true;
            options.overwrite = true;
            fs::create_dir_all(source.clone())?;
            cp_dir(source, target, &options)?;
        } else {
            let mut options = cpo_files::new();
            options.overwrite = true;
            cp_file(source, target, &options)?;
        }
    }
    Ok(())
}
