use crate::CONFIG;
use file_diff::diff;
use log::info;
use std::io::prelude::*;
use std::{io::BufReader, path::PathBuf};
use subprocess::{Popen, PopenConfig, Redirection};

use super::exec_validation::Exectuable;

pub fn test() {
    let p = Popen::create(
        &[Exectuable::new().path, "watch".into()],
        PopenConfig {
            stdout: Redirection::Pipe,
            ..Default::default()
        },
    )
    .unwrap();

    if let Some(output) = &p.stdout {
        let f = BufReader::new(output);
        for line in f.lines() {
            let val = serde_json::from_str::<PathBuf>(line.unwrap().as_str()).unwrap();
            info!("File Update Detected: {:#?}", val);
            if diff(val.to_str().unwrap(), "./Config.yml") {
                CONFIG.lock().unwrap().load_project_conf();
            } else {
                let _ = Popen::create(
                    &[Exectuable::new().path, "build".into()],
                    PopenConfig {
                        stdout: Redirection::Merge,
                        ..Default::default()
                    },
                )
                .unwrap()
                .wait();
            }
        }
    }
}
