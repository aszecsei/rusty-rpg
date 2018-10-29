use fnv::{FnvHashMap, FnvHasher};

use crate::renderer::shader::Shader;
use crate::renderer::texture::Texture;

use std::hash::{Hash, Hasher};
use std::fs::File;
use std::io::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceID(u64);

impl From<&str> for ResourceID {
    fn from(name: &str) -> Self {
        let mut hasher = FnvHasher::default();
        name.hash(&mut hasher);
        ResourceID(hasher.finish())
    }
}

pub struct ResourceManager {
    shaders: FnvHashMap<ResourceID, Shader>,
    textures: FnvHashMap<ResourceID, Texture>
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            shaders: FnvHashMap::default(),
            textures: FnvHashMap::default(),
        }
    }

    pub fn initialize(&mut self) {
        self.load_shader("resources/shaders/sprite.vert", "resources/shaders/sprite.frag", None, ResourceID::from("sprite_shader"));
    }

    pub fn load_shader(&mut self, vertex_shader_file: &str, fragment_shader_file: &str, geometry_shader_file: Option<&str>, name: ResourceID) -> Shader {
        let mut vertex_file = File::open(vertex_shader_file).expect("failed to open file!");
        let mut vertex_shader_source = String::new();
        vertex_file.read_to_string(&mut vertex_shader_source).expect("failed to read file!");

        let mut fragment_file = File::open(fragment_shader_file).expect("failed to open file!");
        let mut fragment_shader_source = String::new();
        fragment_file.read_to_string(&mut fragment_shader_source).expect("failed to read file!");

        let geometry_shader_source = if let Some(geometry_shader_file) = geometry_shader_file {
            let mut geometry_file = File::open(geometry_shader_file).expect("failed to open file!");
            let mut geometry_shader_source = String::new();
            geometry_file.read_to_string(&mut geometry_shader_source).expect("failed to read file!");
            Some(geometry_shader_source)
        } else {
            None
        };

        let mut shader_program = Shader::new();
        if let Some(geometry_shader_source) = geometry_shader_source {
            shader_program.compile(&vertex_shader_source, &fragment_shader_source, Some(&geometry_shader_source));
        } else {
            shader_program.compile(&vertex_shader_source, &fragment_shader_source, None);
        }

        self.shaders.insert(name, shader_program);
        
        shader_program
    }

    pub fn get_shader(&self, name: &ResourceID) -> Shader {
        self.shaders[name]
    }

    pub fn shutdown(&mut self) {
        for shader in self.shaders.values() {
            unsafe {
                gl::DeleteProgram(shader.id);
            }
        }
        for texture in self.textures.values() {
            unsafe {
                gl::DeleteTextures(1, &texture.id);
            }
        }
        self.shaders.clear();
        self.textures.clear();
    }
}