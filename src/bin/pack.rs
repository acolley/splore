extern crate clap;
extern crate image;
extern crate serde;
extern crate serde_json;
extern crate texture_packer;

use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use clap::{Arg, App};
use serde::ser::{Serialize};
use serde_json::Value;
use serde_json::ser::{Serializer};
use serde_json::value::to_value;
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
        .arg(Arg::with_name("BORDER")
            .short("b")
            .long("border")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("TRIM")
            .short("t")
            .long("trim")
            .required(false))
        .get_matches();

    let paths = matches.values_of("TEXTURES").expect("No textures given.");
    let output = matches.value_of("OUTPUT").expect("No output path given.");
    let border = matches.value_of("BORDER").unwrap_or("0").parse::<u32>().ok().expect("Border is not a u32.");
    let trim = matches.is_present("TRIM");

    let mut cfg = TexturePackerConfig::default();
    cfg.allow_rotation = false;
    cfg.border_padding = border;
    cfg.trim = trim;

    let mut packer = TexturePacker::new_skyline(cfg);
    for path in paths.iter().map(|x| Path::new(x)) {
        let texture = ImageImporter::import_from_file(path).unwrap();
        let name = path.file_stem().unwrap().to_os_string().into_string().unwrap();
        packer.pack_own(name, texture);
    }

    let packed = ImageExporter::export(&packer).unwrap();
    let output = Path::new(output);
    let mut outfile = File::create(format!("{}.png", output.to_str().unwrap())).unwrap();

    packed.save(&mut outfile, image::PNG).unwrap();

    let mut json = BTreeMap::new();
    let mut frames = BTreeMap::new();
    for (name, frame) in packer.get_frames().iter() {
        frames.insert(name, (frame.frame.x, frame.frame.y, frame.frame.w, frame.frame.h));
    }
    json.insert("frames", to_value(&frames));
    let json = to_value(&json);

    let mut jsonfile = File::create(format!("{}.json", output.to_str().unwrap())).unwrap();
    let mut serializer = Serializer::pretty(jsonfile);
    json.serialize(&mut serializer);
}