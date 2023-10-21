use anyhow::Result;
use log::trace;
use regex::Regex;
use std::{collections::HashSet, fs::create_dir_all, path::PathBuf, str::FromStr, sync::Arc};

use crate::{
    modules::config::ProjectType,
    utils::interactions::{get_input_string, get_input_string_with_validator, get_multi_input},
    CONFIG, get_confirm,
};

use super::git::Git;


fn scaffold_model(project_path: PathBuf) -> Result<()>{
    match get_confirm!("Do you want to Pull Model Assets from a Github Repo?", true) {
        true => {
            let re =  Regex::new(r"^(https:\/\/)?github\.com\/[A-Za-z0-9_-]+\/[A-Za-z0-9_.-]+\/(tree|blob)\/[A-Za-z0-9_.-]+(\/[A-Za-z0-9_.-]+)*$").unwrap();
            let git_asset_url = get_input_string_with_validator("Enter a Github URL. If you need Assets have a look at (https://github.com/SilentNightSound/GI-Model-Importer-Assets)", None, |input: &String| -> Result<(), &str> {
                if re.is_match(&input) {
                    Ok(())
                } else {
                    Err("This is Not a Valid Github Repo. Try Again.")
                }
            });
            let files = Git::default().load(&PathBuf::from(git_asset_url)).unwrap().download(PathBuf::from("./Test/Source")).expect("Download Failed. Make sure you providing the path to a specific Asset");
        },
        false => (),
    }
    
   
    Ok(())
}


pub fn scaffold(project_path: PathBuf) {
    trace!("In Scaffold Module");
    create_dir_all(&project_path).expect("Failed to Create Project at Specifed Path");
    let project_name = get_input_string(
        "Project Name:",
        Some(
            project_path
                .canonicalize()
                .unwrap()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap(),
        ),
    );

    CONFIG.lock().unwrap().project_name = project_name;

    let authors_binding = get_input_string(
        "Authors (Delimit Multiple Authors with ;):",
        Some(whoami::username().as_str()),
    );
    let authors = authors_binding
        .split(";")
        .map(|f| Arc::from(f))
        .collect::<HashSet<Arc<str>>>();
    CONFIG.lock().unwrap().authors = authors;

    let migoto_path_string =
        get_input_string_with_validator("Path to your 3DMigoto Folder:", None, {
            let mut force = None;
            move |input: &String| -> Result<(), &str> {
                let path = PathBuf::from(&input);
                if (path.exists()
                    && path.read_dir().unwrap().any(|f| {
                        let binding = f.unwrap().path();
                        let file_name = binding.file_name().unwrap().to_str().unwrap();
                        file_name == "d3d11.dll"
                    }))
                    || force.as_ref().map_or(false, |old| old == input)
                {
                    Ok(())
                } else {
                    force = Some(input.clone());
                    Err("This is not a valid 3DMigoto Path; Type the same value again to force use")
                }
            }
        });
    let migoto_path: Option<PathBuf>;
    match migoto_path_string.is_empty() {
        true => migoto_path = None,
        false => migoto_path = Some(PathBuf::from(migoto_path_string)),
    }
    CONFIG.lock().unwrap().migoto_path = migoto_path;

    let project_type = get_multi_input(
        "What Type of Project do you want to make?",
        vec!["3D Model", "Texture", "Shader"],
        None,
    );
    CONFIG.lock().unwrap().project_type = project_type
        .iter()
        .map(|f| ProjectType::from_str(f.as_str()).unwrap())
        .collect();

    let types = CONFIG.lock().unwrap().project_type.clone();
    for project_type in types{
        match project_type {
            ProjectType::Model => scaffold_model(project_path.clone()).expect("Failed to Work with Model Scaffolding"),
            ProjectType::Texture => (),
            ProjectType::Shader => (),
        }

    }

    CONFIG.lock().unwrap().save_project_conf();
}
