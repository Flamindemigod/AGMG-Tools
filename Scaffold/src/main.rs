use std::{path::PathBuf, fs, env::current_dir};
use anyhow::Result;
use clap::Parser;
use fs_extra::dir::{CopyOptions, copy};


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli{
    /// Path to Source Dump
    #[arg(short, long)]
    source: PathBuf,

    /// Copy Symlink Source. Defaults to Making a Copy of Source in Project Root
    #[arg(long, default_value_t = false)]
    symlink_source: bool,

    /// Only Generate the Textures. Mainly to Rebuild Texture Files
    #[arg(long, default_value_t = false)]
    tex_only: bool,   
    
    /// Project Name to use as Project Root
    #[arg(long)]
    name: PathBuf
}



fn make_root_dir(path: PathBuf, root_path: &mut PathBuf) -> Result<()>{
    if !path.is_absolute(){
        *root_path = current_dir()?;
        root_path.push(path);
    }
    else{
        *root_path = path;
    }
    fs::create_dir_all(root_path)?;
    Ok(())
}

#[cfg(target_family = "windows")]
fn symlink(source: PathBuf, target: PathBuf) -> Result<()>{
    use std::os::windows::fs::symlink_dir;
    symlink_dir(source, target)?;
    Ok(())
}

#[cfg(target_family = "unix")]
fn symlink(source: PathBuf, target: PathBuf) -> Result<()>{
    std::os::unix::fs::symlink(source, target)?;
    Ok(())
}


fn copy_source(source_path: PathBuf, root_path: &PathBuf, symlink_source: bool) -> Result<()>{
    let source_folder = root_path.clone().join("Source");
    if symlink_source{ 
        symlink(source_path, source_folder)?;
    } else{
        let mut options = CopyOptions::new();
        options.content_only= true;
         fs::create_dir_all(source_folder.clone())?;
        copy(source_path, source_folder.clone(), &options)?;
    }
    Ok(())
}

fn gen_tex(source_path: PathBuf, root_path: &PathBuf, tex_only: bool){
    let source:PathBuf;
    if tex_only{ 
        source = source_path;
    } else{
        source = root_path.clone().join("Source");
    }
    println!("Tex Source: {:#?}", source);
    todo!("Generate Flat Textures and Alpha Maps")
}

fn main() {
    let args = Cli::parse();
    println!("{:#?}", args);
    let mut root_path = PathBuf::new();
    match make_root_dir(args.name,&mut root_path){
        Ok(_) => (),
        Err(err) => panic!("Error: Failed to Make Root Dir: {}", err)
    }
    println!("Project Root at: {:#?}", root_path);

    if !args.tex_only {
        match copy_source(args.source.clone(), &root_path, args.symlink_source){
            Ok(_) => (),
            Err(err) => panic!("Error: Failed to Copy Source Dir: {}", err)
        }
    }
    gen_tex(args.source.clone(), &root_path, args.tex_only);
}
