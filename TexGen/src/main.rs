use anyhow::Result;
use clap::Parser;
use image::RgbaImage;
use image_dds::{dds_from_image, ImageFormat};
use rayon::prelude::*;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// Path to Target Folder (Generates the Ini Here, Creates the Folder if it doesnt exist)
    #[arg(short, long)]
    target: PathBuf,

    /// Copy Texture Files To Target
    #[arg(short, long, default_value_t = false)]
    cp: bool,

    /// Ini Filename
    #[arg(short, long, default_value = "Mod.ini")]
    ini_name: String,

    /// Files to Use to Build Ini
    #[arg(short, long, num_args(1..), required= true)]
    files: Vec<PathBuf>,
}

#[derive(Debug, Default)]
struct Image {
    data: RgbaImage,
    format: Option<ImageFormat>,
    hash: String,
    filename: String,
}


trait FromStr {
    fn from_str(s: &str) -> Result<ImageFormat>;
}

impl FromStr for ImageFormat {
    fn from_str(s: &str) -> Result<ImageFormat> {
        match s {
            "BC7_UNORM" => Ok(ImageFormat::BC7Unorm),
            "BC7_UNORM_SRGB" => Ok(ImageFormat::BC7Srgb),
            _ => unimplemented!("DXGI Format Not Parsed")
        }
    }
}

trait Load {
    fn load(&mut self, path: &PathBuf);
}

impl Load for Image {
    fn load(&mut self, path: &PathBuf) {
        if path
            .clone()
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .eq("dds")
        {
            let mut reader = File::open(&path).unwrap();
            let dds = ddsfile::Dds::read(&mut reader).unwrap();
            self.data = image_dds::image_from_dds(&dds, 0).unwrap();
        } else {
            self.data = image::open(path).unwrap().to_rgba8();
        }
        self.filename = path.file_stem().unwrap().to_string_lossy().to_string();
        self.format = Some(
            ImageFormat::from_str(self.filename.split_at(9).1)
                .expect("Failed to Get Valid DXGI Format"),
        );
        self.hash = self.filename.split_at(8).0.to_string();
    }
}

fn build_ini(args: &Cli, images: Vec<Image>) -> Result<()> {
    let mod_path = &args.target;
    fs::create_dir_all(&mod_path)?;
    let ini_path = mod_path.join(args.ini_name.as_str());
    let mut file = File::create(&ini_path)?;
    for img in images {
        writeln!(file, "[TextureOverride{:}]", img.hash)?;
        writeln!(file, "hash = {:}", img.hash)?;
        writeln!(file, "this = Resource{:}\n", img.hash)?;

        writeln!(file, "[Resource{:}]", img.hash)?;
        writeln!(file, "filename = {:}.dds\n", img.filename)?;

        let mut filename = img.filename;
        filename.push_str(".dds");
        if args.cp {
            let out_dds = dds_from_image(
                &img.data,
                img.format.unwrap(),
                image_dds::Quality::Slow,
                image_dds::Mipmaps::Disabled,
            )
            .unwrap();
            let mut writer = BufWriter::new(File::create(mod_path.join(filename)).unwrap());
            out_dds.write(&mut writer).unwrap();
        }
    }
    println!("Ini Built at: {:#?}", &ini_path);
    Ok(())
}

fn main() {
    let args = Cli::parse_from(wild::args());
    let mut images: Vec<Image> = Vec::new();
    args.files
        .par_iter()
        .map(|f| {
            let mut img = Image::default();
            img.load(&f.to_path_buf());
            return img;
        })
        .collect_into_vec(&mut images);
    build_ini(&args, images).unwrap();
    println!("Finished Building Ini for {:#?}", &args.files.len());
}
