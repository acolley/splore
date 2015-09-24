
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use glium::backend::Facade;
use glium::texture::{CompressedSrgbTexture2d, Texture};
use image;
use serde_json;
use serde_json::value::Value;

/// A single frame of the TextureAtlas
/// represented in texture space coordinates.
#[derive(Copy, Clone)]
pub struct Frame {
    pub u1: f32,
    pub v1: f32,
    pub u2: f32,
    pub v2: f32,
    pub w: f32, // width in pixels
    pub h: f32, // height in pixels
}

/// The TextureAtlas is a struct that encapsulates
/// the logic in managing a texture that contains 
/// a number of sub-textures.
///
/// The idea behind this is to optimise the number
/// of state changes that need to occur when rendering
/// a large number of triangles that might map to
/// many different textures. In this case a single
/// large texture can contain all smaller textures
/// that are used in a scene and only be bound once.
pub struct TextureAtlas {
    pub texture: CompressedSrgbTexture2d,
    frames: HashMap<String, Frame>
}

impl TextureAtlas {
    pub fn new( 
        texture: CompressedSrgbTexture2d,
        frames: HashMap<String, Frame>) -> TextureAtlas {

        TextureAtlas {
            texture: texture,
            frames : frames
        }
    }

    /// TODO: make this use an asset store of some kind
    /// so that we don't have to load the image in.
    pub fn from_packed<T, F>(
        image_path: T, 
        json_path: T, 
        display: &F) -> TextureAtlas
        where T: AsRef<Path>,
              F: Facade {
        let image = image::open(image_path).unwrap();
        let texture = CompressedSrgbTexture2d::new(display, image).unwrap();

        let mut jsonfile = File::open(json_path).unwrap();
        let ref mut jsonstr = String::new();
        jsonfile.read_to_string(jsonstr);
        let data: Value = serde_json::from_str(jsonstr).unwrap();

        let frames = data.find("frames")
            .unwrap()
            .as_object()
            .unwrap();

        let mut tiles = HashMap::new();
        for (name, frame) in frames.iter() {
            let frame = frame.as_array().unwrap();
            let x = frame[0].as_f64().unwrap();
            let y = frame[1].as_f64().unwrap();
            let w = frame[2].as_f64().unwrap();
            let h = frame[3].as_f64().unwrap();
            let frame = Frame {
                u1: x as f32 / texture.get_width() as f32,
                v1: y as f32 / texture.get_height().unwrap() as f32,
                u2: ((x + w) / texture.get_width() as f64) as f32,
                v2: ((y + h) / texture.get_height().unwrap() as f64) as f32,
                w: w as f32,
                h: h as f32
            };
            tiles.insert(name.clone(), frame);
        }
        TextureAtlas::new(texture, tiles)
    }

    /// Create a TextureAtlas from a collection of images.
    ///
    /// This constructor will load the files itself and
    /// then combine them into one texture. Useful as part of
    /// an initial quick development period but much more
    /// inefficient than pre-processing the combined texture
    /// and loading it in later using `from_packed`.
    // pub fn pack<P: AsRef<Path>>(&self, paths: &[P]) -> TextureAtlas {

    // }

    pub fn get_frame(&self, name: &str) -> Option<&Frame> {
        self.frames.get(name)
    }
}
