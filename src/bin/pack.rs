extern crate clap;
extern crate image;
extern crate texture_packer;

use std::fs::File;
use std::path::Path;

use clap::{Arg, App};
use texture_packer::{TexturePacker, TexturePackerConfig};
use texture_packer::exporter::ImageExporter;
use texture_packer::importer::ImageImporter;

fn main() {
    let matches = App::new("pack")
        .version("1.0")
        .arg(Arg::with_name("TEXTURES")
             .multiple(true)
             .required(true))
        .arg(Arg::with_name("OUTPUT")
             .short("o")
             .long("output")
             .takes_value(true)
             .required(true))
        .get_matches();

    let paths = matches.values_of("TEXTURES").expect("No textures given.");
    let output = matches.value_of("OUTPUT").expect("No output path given.");

    let mut cfg = TexturePackerConfig::default();
    cfg.allow_rotation = false;
    cfg.border_padding = 2;

    let mut packer = TexturePacker::new_skyline(cfg);
    for path in paths.iter().map(|x| Path::new(x)) {
        let texture = ImageImporter::import_from_file(path).unwrap();
        let name = path.file_stem().unwrap().to_os_string().into_string().unwrap();
        packer.pack_own(name, texture);
    }

    let packed = ImageExporter::export(&packer).unwrap();
    let output = Path::new(output);
    let mut outfile = File::create(output).unwrap();

    packed.save(&mut outfile, image::PNG).unwrap();
}