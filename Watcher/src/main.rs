use anyhow::Result;
use clap::Parser;
use fs_extra::dir::{copy, CopyOptions};
use glob::glob;
use globset::{Glob, GlobSet, GlobSetBuilder};
use image::{GrayImage, Luma, RgbaImage};
use image_dds::{
    dds_from_image,
    ImageFormat::{BC7Srgb, BC7Unorm},
};
use itertools::izip;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEvent};
use std::{
    fs::{self, File},
    io::BufWriter,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Path to 3DMigoto's Mod Folder
    #[arg(short, long)]
    mod_folder_path: PathBuf,

    /// Symlink Mod into Mod folder. Defaults to Making a Copy of Mod in ModFolder
    #[arg(long, default_value_t = false)]
    symlink_source: bool,

    /// Ignore Rebuild on Texture Changes.
    #[arg(long, default_value_t = false)]
    disable_tex: bool,

    /// Path to Project.
    #[arg(short, long, default_value = ".")]
    project_path: PathBuf,

    /// Mod Name to Use in Mod Folder
    #[arg(short, long)]
    name: String,
}

#[cfg(target_family = "windows")]
fn symlink(source: PathBuf, target: PathBuf) -> Result<()> {
    use std::os::windows::fs::symlink_dir;
    symlink_dir(source, target)?;
    Ok(())
}

#[cfg(target_family = "unix")]
fn symlink(source: PathBuf, target: PathBuf) -> Result<()> {
    std::os::unix::fs::symlink(source, target)?;
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

fn copy_mod(orig: PathBuf, out: PathBuf, symlink_mod: bool) -> Result<()> {
    if symlink_mod {
        symlink(orig, out)?;
    } else {
        let mut options = CopyOptions::new();
        options.content_only = true;
        options.overwrite = true;
        fs::create_dir_all(out.clone())?;
        copy(orig, out.clone(), &options)?;
    }
    Ok(())
}

fn grab_tex(
    source: PathBuf,
    target: PathBuf,
    image_type: &str,
    split_alpha: bool,
    is_srgb: bool,
) -> Result<()> {
    let out_format;
    if is_srgb {
        out_format = BC7Srgb;
    } else {
        out_format = BC7Unorm;
    }

    let mut glob_pattern = "*".to_owned();
    glob_pattern.push_str(image_type);
    glob_pattern.push_str("*.png");
    let full_glob_pattern = source.join(glob_pattern);
    let mut matched_paths = glob(full_glob_pattern.to_str().expect("Failed to parse str"))?
        .into_iter()
        .map(|p| p.unwrap())
        .collect::<Vec<_>>();
    matched_paths.sort();

    let itterator_step: usize;
    if split_alpha {
        itterator_step = 2;
    } else {
        itterator_step = 1;
    }

    while !matched_paths.is_empty() {
        let mut file_join: Vec<PathBuf> = Vec::new();
        for _ in 0..itterator_step {
            file_join.push(matched_paths.pop().unwrap())
        }
        let _flat_path = &file_join
            .iter()
            .filter(|file| {
                file.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains("Flat.png")
            })
            .collect::<Arc<[_]>>();
        if _flat_path.is_empty() {
            return Ok(());
        }
        let flat_path = _flat_path[0];
        let _file_name = flat_path.file_stem().unwrap().to_string_lossy();
        let file_name = _file_name.strip_suffix("Flat").unwrap();
        let mut flat = image::open(flat_path).unwrap().to_rgba8();

        if split_alpha {
            let alpha = &file_join
                .iter()
                .filter(|file| {
                    file.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .contains("Alpha.png")
                })
                .map(|file: &PathBuf| image::open(file).unwrap().to_luma8())
                .collect::<Arc<[_]>>()[0];
            let mut flat_channels = flat.split_channels();
            flat_channels[3] = alpha.clone();
            flat.join_channels(flat_channels);
        }

        let mut out_file_name = "".to_owned();
        out_file_name.push_str(file_name);
        out_file_name.push_str(".dds");
        let out_dds = dds_from_image(
            &flat,
            out_format,
            image_dds::Quality::Slow,
            image_dds::Mipmaps::Disabled,
        )
        .unwrap();
        let mut writer = BufWriter::new(File::create(target.join(out_file_name)).unwrap());
        out_dds.write(&mut writer).unwrap();
    }

    Ok(())
}

fn gen_tex(root_path: &PathBuf) -> Result<()> {
    let tex_path = root_path.clone().join("Textures");
    let output_path = root_path.clone().join("Output");
    grab_tex(tex_path.clone(), output_path.clone(), "Diffuse", true, true)?;
    grab_tex(
        tex_path.clone(),
        output_path.clone(),
        "LightMap",
        true,
        false,
    )?;
    grab_tex(
        tex_path.clone(),
        output_path.clone(),
        "NormalMap",
        false,
        true,
    )?;
    Ok(())
}

fn rebuild(args: &Cli) -> Result<()> {
    gen_tex(&args.project_path)?;
    copy_mod(
        args.project_path.join("Output"),
        args.mod_folder_path.join(&args.name),
        args.symlink_source,
    )?;
    Ok(())
}

fn on_event_trigger(events: Vec<DebouncedEvent>, matcher: &GlobSet, args: &Cli) -> Result<()> {
    events.iter().for_each(|e| {
        if matcher.matches(&e.path).len() > 0 {
            println!("Starting Rebuild");
            rebuild(args).expect("Failed to rebuild");
            println!("Finished Rebuilding");
        }
    });

    Ok(())
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new("**/Output/*.ini")?);
    if !args.disable_tex {
        builder.add(Glob::new("**/Textures/*.png")?);
    }
    let matcher = builder.build()?;

    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), tx).unwrap();

    debouncer
        .watcher()
        .watch(&args.project_path, RecursiveMode::Recursive)?;
    for res in rx {
        match res {
            Ok(event) => on_event_trigger(event, &matcher, &args)?,
            Err(error) => println!("Error: {error:?}"),
        }
    }
    Ok(())
}
