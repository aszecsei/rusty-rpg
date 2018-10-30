use fnv::{FnvHashMap, FnvHasher};

use crate::renderer::shader::Shader;
use crate::renderer::texture::Texture;

use std::hash::{Hash, Hasher};

use image::GenericImageView;
use resource::{resource, resource_str};

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
        self.load_shader(&resource_str!("../resources/shaders/sprite.vert"), &resource_str!("../resources/shaders/sprite.frag"), None, ResourceID::from("sprite_shader"));
        self.load_shader(&resource_str!("../resources/shaders/font.vert"), &resource_str!("../resources/shaders/font.frag"), None, ResourceID::from("text_shader"));

        self.load_texture(&resource!("../resources/textures/awesomeface.png"), image::ImageFormat::PNG, ResourceID::from("awesome_face"));
    }

    fn load_shader(&mut self, vertex_shader_file: &str, fragment_shader_file: &str, geometry_shader_file: Option<&str>, name: ResourceID) -> Shader {

        let mut shader_program = Shader::new();
        if let Some(geometry_shader_file) = geometry_shader_file {
            shader_program.compile(&vertex_shader_file, &fragment_shader_file, Some(&geometry_shader_file));
        } else {
            shader_program.compile(&vertex_shader_file, &fragment_shader_file, None);
        }

        self.shaders.insert(name, shader_program);
        
        shader_program
    }

    pub fn get_shader(&self, name: ResourceID) -> Shader {
        self.shaders[&name]
    }

    fn load_texture(&mut self, texture_file: &[u8], format: image::ImageFormat, name: ResourceID) -> Texture {
        let mut texture = Texture::new();

        let img = image::load_from_memory_with_format(texture_file, format).expect("failed to load image");
        let (width, height) = img.dimensions();

        texture.from_source_rgba(width, height, &img.flipv().raw_pixels());

        self.textures.insert(name, texture);
        texture
    }

    pub fn get_texture(&self, name: ResourceID) -> Texture {
        self.textures[&name]
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