use std::{collections::HashMap, path::PathBuf, sync::Arc};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
struct Config {
    project_name: String,
    authors: Vec<String>,
    migoto_mod_path: Option<PathBuf>,
    // Key -> Target
    // Value -> Source
    textures: HashMap<String, Arc<[PathBuf]>>,
    scripts: HashMap<String, String>
}

impl Config {
    fn load_project_conf() {
        todo!("need to implement project loading");
    }

    fn save_project_conf(){
        todo!("need to implement project save");
    }
}
