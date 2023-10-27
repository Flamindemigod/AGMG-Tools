use anyhow::Result;
use fs_extra::{
    dir::{copy as cp_dir, CopyOptions as cpo_dir},
    file::{copy as cp_file, CopyOptions as cpo_files},
};
use log::trace;
use std::{fs, path::PathBuf};

#[cfg(target_family = "windows")]
fn symlink(source: &PathBuf, target: &PathBuf) -> Result<()> {
    use fs_extra::{dir::remove as remove_dir, file::remove as remove_file};
    use std::os::windows::fs::{symlink_dir, symlink_file};

    trace!(
        "Symlinking: {:} to {:}",
        source.to_string_lossy(),
        target.to_string_lossy()
    );

    if source.is_dir() {
        if target.exists() {
            remove_dir(target)?;
        }
        symlink_dir(source, target)?;
    } else {
        if target.exists() {
            remove_file(target)?;
        }
        symlink_file(source, target)?;
    }
    Ok(())
}

#[cfg(target_family = "unix")]
fn symlink(source: &PathBuf, target: &PathBuf) -> Result<()> {
    trace!(
        "Symlinking: {:} to {:}",
        source.to_string_lossy(),
        target.to_string_lossy()
    );
    std::os::unix::fs::symlink(source, target)?;
    Ok(())
}

pub fn copy(source: &PathBuf, target: &PathBuf, should_symlink: bool) -> Result<()> {
    if should_symlink {
        symlink(source, target)?;
    } else {
        if source.is_dir() {
            fs::create_dir_all(target)?;
            let mut options = cpo_dir::new();
            options.content_only = true;
            options.overwrite = true;
            cp_dir(source, target, &options)?;
        } else {
            let mut options = cpo_files::new();
            options.overwrite = true;
            cp_file(source, target, &options)?;
        }
    }
    Ok(())
}
