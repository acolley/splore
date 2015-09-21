#[macro_use]
extern crate glium;
extern crate glium_text;
extern crate image;
extern crate nalgebra as na;
extern crate serde_json;

use std::collections::HashMap;
use std::default::Default;
use std::rc::{Rc};

use glium::{IndexBuffer, Program, Surface, VertexBuffer};
use glium::backend::Facade;
use glium::glutin;
use glium::glutin::ElementState::Pressed;
use glium::glutin::Event;
use glium::glutin::VirtualKeyCode;
use glium::index::PrimitiveType;
use glium::texture::{CompressedSrgbTexture2d};
use na::{Iso3, Ortho3, Pnt2, Pnt3, Vec3};
use na::{ToHomogeneous};

use scene::{Scene};
use textureatlas::{Frame, TextureAtlas};
use tilemap::{Tile, TileMap};

mod scene;
mod textureatlas;
mod tilemap;

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
    pub down: bool,
    pub activate: bool
}

fn main() {
    use glium::DisplayBuild;

    let window = glutin::WindowBuilder::new()
        .with_dimensions(640, 480)
        .with_title("splore".into())
        .build_glium()
        .unwrap();

    // let mut tile_uvs = HashMap::new();
    // tile_uvs.insert("grass".into(), Frame { u1: 0.0, v1: 0.0, u2: 1.0, v2: 1.0 });
    // let grass = image::open("resources/grass.png").unwrap();
    // let atlas_texture = CompressedSrgbTexture2d::new(&window, grass).unwrap();
    // let atlas = TextureAtlas::new(
    //     atlas_texture,
    //     tile_uvs);
    let atlas = TextureAtlas::from_packed(
        "resources/overground.png",
        "resources/overground.json",
        &window);
    let mut tiles = Vec::new();
    for x in 0..10 {
        for y in 0..10 {
            tiles.push(OvergroundTile::Grass);
            // if (x + y) % 2 == 0 { 
            //     tiles.push(OvergroundTile::Grass);
            // } else {
            //     tiles.push(OvergroundTile::Dirt);
            // }
        }
    }
    let tilemap = TileMap::new(
        &window,
        10, 10,
        16, 16,
        tiles,
        atlas
    );

    let spritesheet = TextureAtlas::from_packed(
        "resources/spritesheet.png",
        "resources/spritesheet.json",
        &window);
    let mut scene = Scene::new(&window, spritesheet);
    scene.add_sprite("player", &["player"]);
    scene.add_sprite("player2", &["player"]);
    scene.with_sprite_mut("player2", |x| x.set_position(32.0, 32.0));

    let (width, height) = (640.0, 480.0);
    let proj = Ortho3::new(width * 2.0, height * 2.0, -1.0, 1.0);
    // let proj = Ortho3::new(width, height, -1.0, 1.0);
    let mut view = Iso3::new(na::zero(), na::zero());
    let mut focus = Pnt2::new(width / 2.0, height / 2.0);
    // let mut focus = Pnt2::new(0.0, 0.0);

    let mut input = Input {
        left: false,
        right: false,
        up: false,
        down: false,
        activate: false
    };

    'main: loop {
        view.look_at_z(&Pnt3::new(focus.x, focus.y, -1.0), &Pnt3::new(focus.x, focus.y, 0.0), &Vec3::y());
        let viewproj = proj.to_mat() * na::inv(&view.to_homogeneous()).unwrap();

        let mut frame = window.draw();
        frame.clear_color_and_depth((0.0, 0.0, 0.0, 0.0), 1.0);
        tilemap.draw(&mut frame, &viewproj);
        scene.draw(&mut frame, &viewproj);
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
}
