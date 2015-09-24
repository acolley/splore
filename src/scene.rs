
use std::collections::HashMap;
use std::mem;
use std::ops::Deref;

use glium;
use glium::{
    Blend,
    Depth,
    DrawParameters,
    IndexBuffer,
    Program,
    Surface,
    VertexBuffer
};
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

pub enum Sprite {
    Static {
        position: Pnt3<f32>,
        frame: Frame,
    },
    Animated {
        position: Pnt3<f32>,
        frames: Vec<Frame>,
        fps: f32,
        current_frame: usize,
    }
}

impl Sprite {
    #[inline]
    pub fn get_current_frame(&self) -> &Frame {
        match *self {
            Sprite::Static { ref frame, .. } => frame,
            Sprite::Animated { ref frames, current_frame, .. } => {
                frames.get(current_frame)
                    .expect(&format!("Not a valid frame index: `{}`", current_frame))
            },
        }
    }

    #[inline]
    pub fn get_position(&self) -> &Pnt3<f32> {
        match *self {
            Sprite::Static { ref position, .. } => position,
            Sprite::Animated { ref position, .. } => position
        }
    }

    #[inline]
    pub fn set_position(&mut self, x: f32, y: f32) {
        let mut position = match *self {
            Sprite::Static { ref mut position, .. } => position,
            Sprite::Animated { ref mut position, .. } => position,
        };
        position.x = x;
        position.y = y;
    }

    #[inline]
    pub fn set_position_z(&mut self, z: f32) {
        let mut position = match *self {
            Sprite::Static { ref mut position, .. } => position,
            Sprite::Animated { ref mut position, .. } => position,
        };
        position.z = z;
    }
}

pub struct Scene<F> {
    capacity: usize,
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
    ).unwrap()
}

impl<F: Facade + Clone> Scene<F> {
    pub fn new(display: &F, texture: TextureAtlas) -> Scene<F> {
        Scene::with_capacity(display, texture, 50)
    }

    pub fn with_capacity(display: &F, texture: TextureAtlas, n: usize) -> Scene<F> {
        Scene {
            capacity: n,
            texture: texture,
            sprites: HashMap::with_capacity(n),
            program: get_program(display),
            vertex_buffer: VertexBuffer::empty_dynamic(display, 4 * n)
                .ok().expect("Could not create VertexBuffer"),
            index_buffer: IndexBuffer::empty_dynamic(display, PrimitiveType::TrianglesList, 6 * n)
                .ok().expect("Could not create IndexBuffer"),
            display: display.clone(),
        }
    }

    /// Update the animation on any animated sprites
    pub fn update(&mut self, dt: f32) {

    }

    /// Upload the data to the GPU for drawing
    fn upload_data(&mut self) {
        let vstride = mem::size_of::<Vertex>();
        let istride = mem::size_of::<u16>();
        let voffset = 4 * self.sprites.len();
        let ioffset = 6 * self.sprites.len();

        let mut vertices = Vec::with_capacity(voffset);
        let mut indices = Vec::with_capacity(ioffset);
        for (i, sprite) in self.sprites.values().enumerate() {
            let position = sprite.get_position();
            let frame = sprite.get_current_frame();
            let x1 = position.x;
            let x2 = position.x + frame.w;
            let y1 = position.y;
            let y2 = position.y + frame.h;
            vertices.push(Vertex { position: [x1, y1, position.z], texcoords: [frame.u1, frame.v1] });
            vertices.push(Vertex { position: [x1, y2, position.z], texcoords: [frame.u1, frame.v2] });
            vertices.push(Vertex { position: [x2, y2, position.z], texcoords: [frame.u2, frame.v2] });
            vertices.push(Vertex { position: [x2, y1, position.z], texcoords: [frame.u2, frame.v1] });

            let index = (i * 4) as u16;
            indices.push(index+1);
            indices.push(index+2);
            indices.push(index);

            indices.push(index+2);
            indices.push(index);
            indices.push(index+3);
        }

        let mut vertex_slice = self.vertex_buffer
            .slice_mut(0..voffset)
            .expect("Could not take a mutable slice of VertexBuffer");
        let mut index_slice = self.index_buffer
            .slice_mut(0..ioffset)
            .expect("Could not take a mutable slice of IndexBuffer");

        vertex_slice.write(&vertices);
        index_slice.write(&indices);
    }

    pub fn draw<S: Surface>(&mut self, surface: &mut S, viewproj: &Mat4<f32>) {
        self.upload_data();

        let sampled_texture = self.texture.texture.sampled()
            .minify_filter(MinifySamplerFilter::Nearest)
            .magnify_filter(MagnifySamplerFilter::Nearest);
        let uniforms = uniform! {
            matrix: viewproj.clone(),
            tex: sampled_texture
        };

        let vertex_slice = self.vertex_buffer
            .slice(0..self.sprites.len() * 4)
            .expect("Could not take a slice of VertexBuffer");
        let index_slice = self.index_buffer
            .slice(0..self.sprites.len() * 6)
            .expect("Could not take a slice of IndexBuffer");

        let mut params = DrawParameters::default();
        params.blend = Blend::alpha_blending();
        params.depth = Depth {
            test: glium::DepthTest::IfLessOrEqual,
            write: true,
            .. Default::default()
        };
        surface.draw(
            vertex_slice,
            index_slice,
            &self.program,
            &uniforms,
            &params).unwrap();
    }

    /// Extend the Vertex/Index buffers to double
    /// their current capacity.
    fn extend_buffers(&mut self) {
        self.vertex_buffer = VertexBuffer::empty_dynamic(&self.display, 4 * self.sprites.capacity())
            .ok().expect("Could not create VertexBuffer");
        self.index_buffer = IndexBuffer::empty_dynamic(&self.display, PrimitiveType::TrianglesList, 6 * self.sprites.capacity())
            .ok().expect("Could not create IndexBuffer");
    }

    pub fn resize(&mut self) {}

    pub fn trim(&mut self) {}

    /// Add a static Sprite to the Scene
    pub fn add_sprite(&mut self, name: &str, frame: &str) {
        {
            let frame = self.texture.get_frame(frame)
                .expect(&format!("No frame with name: `{}`", frame));
            let sprite = Sprite::Static {
                position : Pnt3::new(0.0, 0.0, 0.0),
                frame : frame.clone(),
            };
            self.sprites.insert(name.to_string(), sprite);
        }

        if self.sprites.capacity() > self.capacity {
            self.capacity = self.sprites.capacity();
            self.extend_buffers();
        }
    }

    pub fn add_sprite_animated(&mut self, name: &str, frames: &[&str]) {

    }

    #[inline]
    pub fn remove_sprite(&mut self, name: &str) {

    }

    #[inline]
    pub fn get_sprite(&self, name: &str) -> Option<&Sprite> {
        self.sprites.get(name)
    }

    #[inline]
    pub fn get_sprite_mut(&mut self, name: &str) -> Option<&mut Sprite> {
        self.sprites.get_mut(name)
    }

    pub fn with_sprite<T>(&self, name: &str, f: T)
        where T: Fn(&Sprite) {
        self.sprites.get(name).map(f);
    }

    pub fn with_sprite_mut<T>(&mut self, name: &str, f: T)
        where T: FnMut(&mut Sprite) {
        self.sprites.get_mut(name).map(f);
    }

    // TODO: add iterator over all Sprites
}