use std::{path::PathBuf, fs::{self, File}, env::current_dir};
use anyhow::Result;
use clap::Parser;
use fs_extra::dir::{CopyOptions, copy};
use glob::glob;
use image::{RgbaImage, GrayImage, Luma};
use itertools::izip;

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
        options.overwrite=true;
         fs::create_dir_all(source_folder.clone())?;
        copy(source_path, source_folder.clone(), &options)?;
    }
    Ok(())
}

pub trait Decompose {
    fn split_channels(&self) -> [GrayImage; 4];
}


impl Decompose for RgbaImage {
    fn split_channels(&self) -> [GrayImage; 4] {
        let mut red = GrayImage::new(self.width(), self.height());
        let mut green = red.clone();
        let mut blue = red.clone();
        let mut alpha = red.clone();
        izip!(
            red.pixels_mut(),
            green.pixels_mut(),
            blue.pixels_mut(),
            alpha.pixels_mut(),
            self.pixels()
        )
        .for_each(|(r, g, b, a, rgba)| {
            *r = Luma([rgba[0]]);
            *g = Luma([rgba[1]]);
            *b = Luma([rgba[2]]);
            *a = Luma([rgba[3]]);
        });

        [red, green, blue, alpha]
    }
}

pub trait Compose {
    fn join_channels(&mut self, channels: [GrayImage; 4]) -> ();
}

impl Compose for RgbaImage {
    fn join_channels(&mut self, channels: [GrayImage; 4]) -> () {
        
        izip!(
            channels[0].pixels(),
            channels[1].pixels(),
            channels[2].pixels(),
            channels[3].pixels(),
            self.pixels_mut()
        )
        .for_each(|(r, g, b, a, rgba)| {
            rgba[0] = r[0];
            rgba[1] = g[0];
            rgba[2] = b[0];
            rgba[3] = a[0];
        });

    }
}

fn grab_tex(source: PathBuf,target: PathBuf,  image_type: &str, split_alpha: bool) -> Result<()>{
    let mut glob_pattern = "*".to_owned();
    glob_pattern.push_str(image_type);
    glob_pattern.push_str(".dds");
    let full_glob_pattern = source.join(glob_pattern);
    for entry in glob(full_glob_pattern.to_str().expect("Failed to parse str"))?{
        let path = entry?.clone();
        let filename = path.file_stem().expect("Failed to get Filename").to_str().expect("Failed to parse str");
        let mut reader = File::open(&path).unwrap();

        let dds = ddsfile::Dds::read(&mut reader).unwrap();
        let mut image:RgbaImage = image_dds::image_from_dds(&dds, 0).unwrap();
        let mut rgba = image.split_channels();
        
        let out_file = target.clone();
        
        if split_alpha{
            let mut target_file_name = "".to_owned();
            target_file_name.push_str(filename);
            target_file_name.push_str("Alpha");
            target_file_name.push_str(".png");
            let alpha = &rgba[3];
            alpha.save(out_file.clone().join(target_file_name))?;
        }
        let mut target_file_name = "".to_owned();
        target_file_name.push_str(filename);
        target_file_name.push_str("Flat");
        target_file_name.push_str(".png");
        rgba[3].pixels_mut().for_each(|p| p[0] = 255);
        image.join_channels(rgba);
        image.save(out_file.clone().join(target_file_name))?;
    }
    Ok(())
}


fn gen_tex(source_path: PathBuf, root_path: &PathBuf, tex_only: bool) -> Result<()>{
    let source:PathBuf;
    if tex_only{ 
        source = source_path;
    } else{
        source = root_path.clone().join("Source");
    }
    let tex_path = root_path.clone().join("Textures");
    fs::create_dir_all(tex_path.clone())?;

    grab_tex(source.clone(), tex_path.clone(), "Diffuse", true)?;
    grab_tex(source.clone(), tex_path.clone(), "LightMap", true)?;
    grab_tex(source.clone(), tex_path.clone(), "NormalMap", false)?;
    Ok(())

    // todo!("Generate Flat Textures and Alpha Maps")
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
    match gen_tex(args.source.clone(), &root_path, args.tex_only){
        Ok(_) => (),
        Err(err) => panic!("Error: Failed to Generate Textures: {}", err)
    }
}
