use std::process::exit;

use log::error;

use crate::VERSION;

pub fn update() {
    match VERSION.has_update() {
        Ok(update) => update().expect("Update Failed"),
        Err(error) => {
            error!("{error}");
            exit(1);
        }
    }
}
