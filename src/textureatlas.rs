
use std::collections::HashMap;

use glium::texture::{CompressedSrgbTexture2d};

pub struct TextureAtlas {
    pub tile_width: u32,
    pub tile_height: u32,
    pub texture: CompressedSrgbTexture2d,
    tiles: HashMap<String, (f32, f32, f32, f32)>
}

impl TextureAtlas {
    pub fn new(
        tile_width: u32, 
        tile_height: u32, 
        texture: CompressedSrgbTexture2d,
        tiles: HashMap<String, (f32, f32, f32, f32)>) -> TextureAtlas {

        TextureAtlas {
            tile_width : tile_width,
            tile_height : tile_height,
            texture: texture,
            tiles : tiles
        }
    }

    pub fn get_uvs(&self, name: &str) -> Option<&(f32, f32, f32, f32)> {
        self.tiles.get(name)
    }
}