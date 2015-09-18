
use std::collections::HashMap;

use glium::texture::{CompressedSrgbTexture2d};

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
    tiles: HashMap<String, (f32, f32, f32, f32)>
}

impl TextureAtlas {
    pub fn new( 
        texture: CompressedSrgbTexture2d,
        tiles: HashMap<String, (f32, f32, f32, f32)>) -> TextureAtlas {

        TextureAtlas {
            texture: texture,
            tiles : tiles
        }
    }

    pub fn get_uvs(&self, name: &str) -> Option<&(f32, f32, f32, f32)> {
        self.tiles.get(name)
    }
}
