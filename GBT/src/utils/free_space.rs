use std::path::Path;
use normpath::PathExt;
use sysinfo::{System, SystemExt, DiskExt};
use log::trace;
// TODO: support for relative paths

/// Get available free disk space by specified path
/// 
/// Can return `None` if path is not prefixed by any available disk
pub fn available(path: impl AsRef<Path>) -> Option<u64> {
    let mut system = System::new();

    system.refresh_disks_list();
    system.refresh_disks();

    system.sort_disks_by(|a, b| {
        let a = a.mount_point().as_os_str().len();
        let b = b.mount_point().as_os_str().len();

        a.cmp(&b).reverse()
    });

    trace!("{:#?}", path.as_ref());
    let path = path.as_ref().parent().unwrap().normalize().unwrap();
    for disk in system.disks() {
        if path.starts_with(disk.mount_point()) {
            return Some(disk.available_space());
        }
    }

    None
}

