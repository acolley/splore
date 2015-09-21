
use std::collections::HashMap;
use std::mem;
use std::ops::Deref;

use glium::{IndexBuffer, Program, Surface, VertexBuffer};
use glium::backend::Facade;
use glium::buffer::BufferSlice;
use glium::index::PrimitiveType;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};
use na;
use na::{Mat4, Pnt3};

use textureatlas::{Frame, TextureAtlas};

#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    texcoords: [f32; 2]
}

implement_vertex!(Vertex, position, texcoords);

pub struct Sprite {
    position: Pnt3<f32>,
    frames: Vec<Frame>,
}

impl Sprite {
    #[inline]
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position.x = x;
        self.position.y = y;
    }

    #[inline]
    pub fn set_position_z(&mut self, z: f32) {
        self.position.z = z;
    }
}

pub struct Scene<F> {
    texture: TextureAtlas,
    sprites: HashMap<String, Sprite>,
    program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
    index_buffer: IndexBuffer<u16>,
    display: F
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
                in vec3 position;
                in vec2 texcoords;
                out vec2 v_texcoords;
                void main() {
                    gl_Position = matrix * vec4(position, 1.0);
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
                attribute vec3 position;
                attribute vec2 texcoords;
                varying vec2 v_texcoords;
                void main() {
                    gl_Position = matrix * vec4(position, 1.0);
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
                attribute lowp vec3 position;
                attribute lowp vec2 texcoords;
                varying lowp vec2 v_texcoords;
                void main() {
                    gl_Position = matrix * vec4(position, 1.0);
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

impl<F: Facade + Clone> Scene<F> {
    pub fn new(display: &F, texture: TextureAtlas) -> Scene<F> {
        Scene::with_capacity(display, texture, 50)
    }

    pub fn with_capacity(display: &F, texture: TextureAtlas, n: usize) -> Scene<F> {
        Scene {
            texture : texture,
            sprites : HashMap::new(),
            program : get_program(display),
            vertex_buffer : VertexBuffer::empty_dynamic(display, 4 * n)
                .ok().expect("Could not create VertexBuffer"),
            index_buffer : IndexBuffer::empty_dynamic(display, PrimitiveType::TrianglesList, 6 * n)
                .ok().expect("Could not create IndexBuffer"),
            display : display.clone()
        }
    }

    /// Update the animation on any animated sprites
    pub fn update(&mut self, dt: f32) {

    }

    pub fn draw<S: Surface>(&self, surface: &mut S, viewproj: &Mat4<f32>) {
        let sampled_texture = self.texture.texture.sampled()
            .minify_filter(MinifySamplerFilter::Nearest)
            .magnify_filter(MagnifySamplerFilter::Nearest);
        let uniforms = uniform! {
            matrix: viewproj.clone(),
            tex: sampled_texture
        };
        let vertex_slice = self.vertex_buffer
            .slice(0..self.sprites.len() * 4 * mem::size_of::<Vertex>())
            .expect("Could not take a slice of VertexBuffer.");
        let index_slice = self.index_buffer
            .slice(0..self.sprites.len() * 6 * mem::size_of::<u16>())
            .expect("Could not take a slice of IndexBuffer");
        surface.draw(
            vertex_slice,
            index_slice,
            &self.program,
            &uniforms,
            &Default::default()).unwrap();
    }

    /// Extend the Vertex/Index buffers to double
    /// their current capacity.
    // TODO: return Result indicating whether extending
    // the buffers was successful or not.
    fn extend_buffers(&mut self) {
        let mut vertex_buffer = VertexBuffer::empty_dynamic(
            &self.display,
            self.sprites.len() * 2 * 4)
            .ok().expect("Could not create VertexBuffer");
        let mut index_buffer = IndexBuffer::empty_dynamic(
            &self.display,
            PrimitiveType::TrianglesList,
            self.sprites.len() * 2 * 6)
            .ok().expect("Could not create IndexBuffer");
        {
            let vertex_slice = vertex_buffer
                .deref()
                .slice(0..self.sprites.len() * 4)
                .expect("Could not take a slice of VertexBuffer");
            let index_slice = index_buffer
                .deref()
                .slice(0..self.sprites.len() * 6)
                .expect("Could not take a slice of IndexBuffer");
            self.vertex_buffer.copy_to(vertex_slice);
            self.index_buffer.copy_to(index_slice);
        }
        self.vertex_buffer = vertex_buffer;
        self.index_buffer = index_buffer;
    }

    pub fn resize(&mut self) {}

    pub fn trim(&mut self) {}

    // TODO: return Result indicating whether creating
    // the Vertex- and Index-buffers was successful or not.
    pub fn add_sprite(&mut self, name: &str, frames: &[&str]) {
        assert!(frames.len() > 0);
        let frames: Vec<Frame> = frames.iter()
            .map(|x| {
                self.texture.get_frame(x)
                    .expect(&format!("No frame with name: {}", x))
                    .clone()
            })
            .collect();

        let vstride = mem::size_of::<Vertex>();
        let istride = mem::size_of::<u16>();
        let voffset = 4 * self.sprites.len();
        let ioffset = 6 * self.sprites.len();

        if self.vertex_buffer.get_size() < voffset * vstride {
            self.extend_buffers();
        }

        // FIXME: The way this works limits this to not supporting removing
        // Sprites from the Scene. Ideally this would be using instancing and
        // have small Vertex and Index buffers, where each instance would have
        // a set of texture coordinates and world transform that defines how
        // it should be drawn.
        {
            let vertex_slice = self.vertex_buffer
                .slice(voffset..voffset + 4)
                .expect("Could not take a slice of VertexBuffer.");
            let frame = frames[0];
            vertex_slice.write(&[
                Vertex { position: [0.0, 0.0, 0.0], texcoords: [frame.u1, frame.v1] },
                Vertex { position: [0.0, 16.0, 0.0], texcoords: [frame.u1, frame.v2] },
                Vertex { position: [16.0, 16.0, 0.0], texcoords: [frame.u2, frame.v2] },
                Vertex { position: [16.0, 0.0, 0.0], texcoords: [frame.u2, frame.v1] }
            ]);
            let index_slice = self.index_buffer
                .slice(ioffset..ioffset + 6)
                .expect("Could not take a slice of IndexBuffer.");
            let index = (self.sprites.len() * 4) as u16;
            index_slice.write(&[
                index + 1, index + 2, index,
                index + 2, index, index + 3 
            ]);
        }

        let sprite = Sprite {
            position : Pnt3::new(0.0, 0.0, 0.0),
            frames : frames
        };
        self.sprites.insert(name.to_string(), sprite);
    }

    #[inline]
    pub fn get_sprite<S: AsRef<str>>(&self, name: S) -> Option<&Sprite> {
        self.sprites.get(name.as_ref())
    }

    #[inline]
    pub fn get_sprite_mut<S: AsRef<str>>(&mut self, name: S) -> Option<&mut Sprite> {
        self.sprites.get_mut(name.as_ref())
    }
}