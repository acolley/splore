
use glium::{IndexBuffer, Program, Surface, VertexBuffer};
use glium::backend::Facade;
use glium::index::PrimitiveType;
use na::{Mat4};

use textureatlas::TextureAtlas;

pub struct TileMap<T>
    where T: Default + Tile {
    pub width: usize,
    pub height: usize,
    tiles: Vec<T>,
    pub atlas: TextureAtlas,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub index_buffer: IndexBuffer<u16>,
    program: Program
}

pub trait Tile {
    fn name<'a>(&'a self) -> &'a str;
}

fn get_index(x: u16, y: u16, width: u16) -> u16 {
    (x + y * width) * 3 + x + y * width
}

// TODO: propagate error
fn get_program<F>(display: &F) -> Program
    where F: Facade {
    // compiling shaders and linking them together
    program!(display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec2 texcoords;
                out vec2 v_texcoords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_texcoords = texcoords;
                }
            ",

            fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_texcoords;
                out vec4 f_color;
                void main() {
                    f_color = texture(tex, v_texcoords);
                }
            "
        },

        110 => {  
            vertex: "
                #version 110
                uniform mat4 matrix;
                attribute vec2 position;
                attribute vec2 texcoords;
                varying vec2 v_texcoords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_texcoords = texcoords;
                }
            ",

            fragment: "
                #version 110
                uniform sampler2D tex;
                varying vec2 v_texcoords;
                void main() {
                    gl_FragColor = texture2D(tex, v_texcoords);
                }
            ",
        },

        100 => {  
            vertex: "
                #version 100
                uniform lowp mat4 matrix;
                attribute lowp vec2 position;
                attribute lowp vec2 texcoords;
                varying lowp vec2 v_texcoords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_texcoords = texcoords;
                }
            ",

            fragment: "
                #version 100
                uniform lowp sampler2D tex;
                varying lowp vec2 v_texcoords;
                void main() {
                    gl_FragColor = texture2D(tex, v_texcoords);
                }
            ",
        },
    ).unwrap()
}

impl<T: Default + Tile> TileMap<T> {
    // TODO: return Result<TileMap<T>> so we can propagate construction errors upwards
    // TODO: have TileMap handle its own drawing so that it can own a program and associated
    // shaders
    pub fn new<F>(display: &F, width: usize, height: usize, tiles: Vec<T>, atlas: TextureAtlas) -> TileMap<T>
        where F: Facade {

        assert!(width * height == tiles.len(), "width * height does not equal length of tiles Vec");

        let mut vertices = Vec::with_capacity(width * height * 4);
        let mut indices = Vec::with_capacity(width * height * 6);
        for y in 0..height {
            for x in 0..width {
                let tile_index = width * y + x;
                let tile = tiles.get(tile_index)
                    .expect(&format!("No tile found at index `{}`", tile_index));
                let name = tile.name();
                let &(u1, v1, u2, v2) = atlas.get_uvs(name)
                    .expect(&format!("Could not get uvs from atlas with name `{}`", name));
                let x1 = x as f32 * atlas.tile_width as f32;
                let x2 = x1 + atlas.tile_width as f32;
                let y1 = y as f32 * atlas.tile_height as f32;
                let y2 = y1 + atlas.tile_height as f32;
                vertices.push(Vertex { position: [x1, y1], texcoords: [u1, v1] });
                vertices.push(Vertex { position: [x1, y2], texcoords: [u1, v2] });
                vertices.push(Vertex { position: [x2, y2], texcoords: [u2, v2] });
                vertices.push(Vertex { position: [x2, y1], texcoords: [u2, v1] });
                let index = get_index(x as u16, y as u16, width as u16);
                // first triangle
                indices.push(index + 1);
                indices.push(index + 2);
                indices.push(index);

                // second triangle
                indices.push(index + 2);
                indices.push(index);
                indices.push(index + 3);
            }
        }

        let vertex_buffer = VertexBuffer::new(display, &vertices)
            .ok().expect("Could not create TileMap VertexBuffer");
        let index_buffer = IndexBuffer::new(display, PrimitiveType::TrianglesList, &indices)
            .ok().expect("Could not create TileMap IndexBuffer");

        TileMap {
            width : width,
            height : height,
            tiles : tiles,
            atlas : atlas,
            vertex_buffer : vertex_buffer,
            index_buffer : index_buffer,
            program : get_program(display)
        }
    }

    /// Get the tile at the given indices
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.tiles.get(self.width * y + x)
    }

    /// Iterate over the tiles in row order.
    pub fn iter(&self) -> ::std::slice::Iter<T> {
        self.tiles.iter()
    }

    pub fn draw<S>(&self, surface: &mut S, viewproj: Mat4<f32>) 
        where S: Surface {
        let uniforms = uniform! {
            matrix: viewproj,
            tex: &self.atlas.texture
        };
        surface.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &self.program,
            &uniforms,
            &Default::default()).unwrap();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 2],
    texcoords: [f32; 2]
}

implement_vertex!(Vertex, position, texcoords);