#[macro_use]
extern crate glium;
extern crate image;
extern crate nalgebra as na;

use std::collections::HashMap;
use std::default::Default;
use std::rc::{Rc};

use glium::{IndexBuffer, Surface, VertexBuffer};
use glium::backend::Facade;
use glium::glutin;
use glium::glutin::ElementState::Pressed;
use glium::glutin::Event;
use glium::glutin::VirtualKeyCode;
use glium::index::PrimitiveType;
use glium::texture::{CompressedSrgbTexture2d};
use na::{Iso3, Ortho3, Pnt2, Pnt3, Vec3};
use na::{ToHomogeneous};

pub struct TileMap<T>
    where T: Default + Tile {
    pub width: usize,
    pub height: usize,
    tiles: Vec<T>,
    pub atlas: TextureAtlas,
    pub vertex_buffer: VertexBuffer<Vertex>,
    pub index_buffer: IndexBuffer<u16>
}

fn get_index(x: u16, y: u16, width: u16) -> u16 {
    (x + y * width) * 3 + x + y * width
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
        println!("{:?}", vertices);
        println!("{:?}", indices);
        println!("{}", vertices.len());

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
            index_buffer : index_buffer
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.tiles.get(self.width * y + x)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.tiles.get_mut(self.width * y + x)
    }

    /// Iterate over the tiles in row order.
    pub fn iter(&self) -> ::std::slice::Iter<T> {
        self.tiles.iter()
    }

    /// Iterate mutably over the tiles in row order.
    pub fn iter_mut(&mut self) -> ::std::slice::IterMut<T> {
        self.tiles.iter_mut()
    }
}

pub trait Tile {
    fn name<'a>(&'a self) -> &'a str;
}

pub enum OvergroundTile {
    Dirt,
    Grass
}

impl Default for OvergroundTile {
    fn default() -> OvergroundTile {
        OvergroundTile::Dirt
    }
}

impl Tile for OvergroundTile {
    fn name<'a>(&'a self) -> &'a str {
        match *self {
            OvergroundTile::Dirt => "dirt",
            OvergroundTile::Grass => "grass"
        }
    }
}

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

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 2],
    texcoords: [f32; 2]
}

implement_vertex!(Vertex, position, texcoords);

pub struct Mesh<V>
    where V: glium::Vertex {
    vertices: VertexBuffer<V>
}

// impl<V: glium::Vertex> Mesh<V> {
//     pub fn new(vertices: &[Vertex]) -> Mesh<V> {
//         Mesh {
//             vertices: vertices
//         }
//     }

//     // pub fn draw<S>(&self, surface: &mut S)
//     //     where S: glium::Surface {
//     //     surface.draw(&self.vertices)
//     // }
// }

struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool
}

fn main() {
    use glium::DisplayBuild;

    let window = glutin::WindowBuilder::new()
        .with_dimensions(640, 480)
        .with_title("splore".into())
        .build_glium()
        .unwrap();

    let image = image::open("resources/grass.png").unwrap();
    let texture = CompressedSrgbTexture2d::new(&window, image).unwrap();

    let vertex_buffer = glium::VertexBuffer::new(
        &window,
        &[
            Vertex { position: [0.0, 0.0], texcoords: [0.0, 0.0] },
            Vertex { position: [0.0,  16.0], texcoords: [0.0, 1.0] },
            Vertex { position: [16.0, 16.0], texcoords: [1.0, 1.0] },
            Vertex { position: [16.0, 0.0], texcoords: [1.0, 0.0] },

            Vertex { position: [16.0, 0.0], texcoords: [0.0, 0.0] },
            Vertex { position: [16.0,  16.0], texcoords: [0.0, 1.0] },
            Vertex { position: [32.0, 16.0], texcoords: [1.0, 1.0] },
            Vertex { position: [32.0, 0.0], texcoords: [1.0, 0.0] },
        ]).unwrap();

    let index_buffer = IndexBuffer::new(
        &window,
        PrimitiveType::TrianglesList,
        &[1 as u16, 2, 0, 2, 0, 3, 
          5, 6, 4, 6, 4, 7]).unwrap();

    let mut tile_uvs = HashMap::new();
    tile_uvs.insert("grass".into(), (0.0, 0.0, 1.0, 1.0));
    let image = image::open("resources/grass.png").unwrap();
    let atlas_texture = CompressedSrgbTexture2d::new(&window, image).unwrap();
    let atlas = TextureAtlas::new(
        16, 16,
        atlas_texture,
        tile_uvs);
    let mut tiles = Vec::new();
    for _ in 0..10 {
        for _ in 0..10 {
            tiles.push(OvergroundTile::Grass);
        }
    }
    let tilemap = TileMap::new(
        &window,
        10,
        10,
        tiles,
        atlas
    );

    // compiling shaders and linking them together
    let program = program!(&window,
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
    ).unwrap();

    let (width, height) = (640.0, 480.0);
    let proj = Ortho3::new(width, height, -1.0, 1.0);
    let mut view = Iso3::new(na::zero(), na::zero());
    // let mut focus = Pnt2::new(width / 2.0, height / 2.0);
    let mut focus = Pnt2::new(0.0, 0.0);

    let mut input = Input { left: false, right: false, up: false, down: false };

    'main: loop {
        view.look_at_z(&Pnt3::new(-focus.x, focus.y, 1.0), &Pnt3::new(-focus.x, focus.y, 0.0), &Vec3::y());
        let viewproj = proj.to_mat() * na::inv(&view.to_homogeneous()).unwrap();

        let tex = &tilemap.atlas.texture;
        let uniforms = uniform! {
            matrix: viewproj,
            // matrix: [[1.0, 0.0, 0.0, 0.0],
            //          [0.0, 1.0, 0.0, 0.0],
            //          [0.0, 0.0, 1.0, 0.0],
            //          [0.0, 0.0, 0.0, 1.0f32]],
            // tex: &texture
            tex: tex
        };

        let mut frame = window.draw();
        frame.clear_color(0.0, 0.0, 0.0, 0.0);
        // frame.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &Default::default()).unwrap();
        frame.draw(
            &tilemap.vertex_buffer,
            &tilemap.index_buffer,
            &program,
            &uniforms,
            &Default::default()).unwrap();
        frame.finish().unwrap();

        let speed = 3.0;
        for event in window.poll_events() {
            match event {
                Event::Closed => break 'main,
                Event::KeyboardInput(state, _, Some(VirtualKeyCode::Escape)) => if state == Pressed {
                    break 'main;
                },
                Event::KeyboardInput(state, _, Some(VirtualKeyCode::Right)) => input.right = state == Pressed,
                Event::KeyboardInput(state, _, Some(VirtualKeyCode::Left)) => input.left = state == Pressed,
                Event::KeyboardInput(state, _, Some(VirtualKeyCode::Up)) => input.up = state == Pressed,
                Event::KeyboardInput(state, _, Some(VirtualKeyCode::Down)) => input.down = state == Pressed,
                _ => {}
            }
        }

        if input.right {
            focus.x += speed;
        } else if input.left {
            focus.x -= speed;
        }

        if input.up {
            focus.y += speed;
        } else if input.down {
            focus.y -= speed;
        }
    }

    // let mut map: Map<Tile> = Map::new(10, 10);
    // *map.get_mut(1, 1).unwrap() = Tile::Grass;
}
