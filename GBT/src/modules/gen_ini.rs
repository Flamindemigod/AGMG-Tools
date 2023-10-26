use anyhow::Result;
use itertools::Itertools;
use log::info;
use std::path::PathBuf;

use crate::{
    utils::ini::{Ini, IniSegment},
    CONFIG,
};

use super::config::ProjectType;

fn gen_tex_ini() -> Result<()> {
    let texture_units = CONFIG.lock().unwrap().texture_textures.to_owned();
    let mod_path = PathBuf::from("./Output");
    let project_name = CONFIG.lock().unwrap().project_name.to_owned();
    let ini_path = mod_path.join(format!("{:}_Texture.ini", project_name));

    let mut ini = Ini::new(
        project_name.as_str(),
        CONFIG.lock().unwrap().authors.iter().join(", "),
    );

    for (hash, _) in texture_units {
        ini = ini
            .add_override(
                IniSegment::new(format!("TextureOverride{:}", hash).as_str())
                    .add_field("hash", &hash.as_str())
                    .add_field("this", format!("Resource{:}", hash).as_str()),
            )
            .add_resource(
                IniSegment::new(format!("Resource{:}", hash).as_str())
                    .add_field("filename", format!("{:}.dds", hash).as_str()),
            );
    }
    ini.export(&ini_path)?;
    info!("Texture Ini Built at: {:#?}", &ini_path);
    Ok(())
}

pub fn gen_ini() {
    let project_types = CONFIG.lock().unwrap().project_type.to_owned();
    for project_type in project_types {
        match project_type {
            // INFO: Should be Implemented within gRPC Module;
            ProjectType::Model => (),
            ProjectType::Texture => gen_tex_ini().expect("Generate Ini"),
            ProjectType::Shader => (),
        }
    }
}
