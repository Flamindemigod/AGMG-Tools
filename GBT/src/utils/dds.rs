use std::{fs::{File, self}, io::BufWriter, path::PathBuf};

use anyhow::Result;
use image::{GrayImage, Luma, RgbaImage};
use image_dds::{dds_from_image, dds_image_format};
use itertools::izip;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use log::{trace, info};
use crate::modules::config::{DDSFormat, TexUnit};

trait Decompose {
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

trait Compose {
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

pub fn generate_tex_split(source_dds: PathBuf, target_folder: PathBuf) -> Result<TexUnit> {
    info!("Generating Texture Splits for {:}", &source_dds.file_name().unwrap().to_str().unwrap());
    let filename = source_dds
        .file_stem()
        .expect("Failed to get Filename")
        .to_str()
        .expect("Failed to parse str")
        .to_string();
    fs::create_dir_all(&target_folder)?;
    let mut reader = File::open(&source_dds).expect("Failed to Open Image");
    let dds = ddsfile::Dds::read(&mut reader).unwrap();
    let mut image: RgbaImage = image_dds::image_from_dds(&dds, 0).unwrap();
    let format = DDSFormat::from(dds_image_format(&dds).expect("Failed to Get DDS Format"));
    let mut rgba = image.split_channels();

    let mut files = vec![];
    if filename.contains("Diffuse") || filename.contains("LightMap") {
        let mut target_file_name = "".to_owned();
        target_file_name.push_str(&filename);
        target_file_name.push_str("Alpha");
        target_file_name.push_str(".png");
        let alpha = &rgba[3];
        let target_file_path = target_folder.clone().join(target_file_name);
        alpha.save(target_file_path.clone())?;
        files.push(target_file_path.clone());
        drop(target_file_path);
    }
    let mut target_file_name = "".to_owned();
    target_file_name.push_str(&filename);
    target_file_name.push_str("Flat");
    target_file_name.push_str(".png");
    rgba[3].pixels_mut().for_each(|p| p[0] = 255);
    image.join_channels(rgba);
    let target_file_path = target_folder.clone().join(target_file_name);
    image.save(target_file_path.clone())?;
    files.push(target_file_path.clone());
    drop(target_file_path);

    let tex_unit = TexUnit {
        encoding: format,
        paths: files.into(),
    };
    trace!("Generated Texture Splits for {:} as {:#?}", &source_dds.file_name().unwrap().to_str().unwrap(), tex_unit);
    Ok(tex_unit)
}

pub fn build_from_tex_unit(tex_unit: TexUnit, output_file_path: PathBuf) -> Result<()> {
    let mut flat = tex_unit
        .paths
        .par_iter()
        .filter(|f| {
            f.file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .contains("Flat")
        })
        .map(|f| image::open(f).unwrap().to_rgba8())
        .collect::<Vec<_>>()
        .pop().expect("No Flat Found");
    if tex_unit.paths.len() > 1 {
        let alpha = tex_unit
            .paths
            .par_iter()
            .filter(|f| {
                f.file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
                    .contains("Alpha")
            })
            .map(|f| image::open(f).unwrap().to_luma8())
            .collect::<Vec<_>>()
            .pop().expect("No Alpha Found");
        let mut flat_channels = flat.split_channels();
        flat_channels[3] = alpha;
        flat.join_channels(flat_channels);
    }
    let out_dds = dds_from_image(
        &flat,
        tex_unit.encoding.into(),
        image_dds::Quality::Slow,
        image_dds::Mipmaps::Disabled,
    )
    .unwrap();
    let mut writer = BufWriter::new(File::create(output_file_path)?);
    out_dds.write(&mut writer).unwrap();
    Ok(())
}
