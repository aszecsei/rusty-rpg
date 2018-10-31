use fnv::{FnvHashMap, FnvHasher};

use std::hash::{Hash, Hasher};
use image::GenericImageView;
use resource::{resource};

use vulkano::sync::GpuFuture;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceID(u64);

impl From<&str> for ResourceID {
    fn from(name: &str) -> Self {
        let mut hasher = FnvHasher::default();
        name.hash(&mut hasher);
        ResourceID(hasher.finish())
    }
}

pub struct ResourceManager {
    textures: FnvHashMap<ResourceID, std::sync::Arc<vulkano::image::immutable::ImmutableImage<vulkano::format::R8G8B8A8Srgb>>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        ResourceManager {
            textures: FnvHashMap::default(),
        }
    }

    pub fn load_resources(&mut self, device: &std::sync::Arc<vulkano::device::Device>, queue: &std::sync::Arc<vulkano::device::Queue>) -> Box<GpuFuture> {
        let mut gpu_future = Box::new(vulkano::sync::now(device.clone())) as Box<GpuFuture>;

        gpu_future = self.load_texture(&resource!("../resources/textures/awesomeface.png"), image::ImageFormat::PNG, queue, gpu_future, ResourceID::from("awesome_face"));

        gpu_future
    }

    fn load_texture(&mut self, texture_data: &[u8], format: image::ImageFormat, queue: &std::sync::Arc<vulkano::device::Queue>, old_future: Box<GpuFuture>, name: ResourceID) -> Box<GpuFuture> {
        let image = image::load_from_memory_with_format(texture_data, format).expect("failed to load image");
        let (width, height) = image.dimensions();
        let image_data = image.flipv().raw_pixels().clone();

        let (texture, tex_future) = {
            vulkano::image::immutable::ImmutableImage::from_iter(
                image_data.iter().cloned(),
                vulkano::image::Dimensions::Dim2d { width, height },
                vulkano::format::R8G8B8A8Srgb,
                queue.clone()).unwrap()
        };

        self.textures.insert(name, texture);

        Box::new(old_future.join(tex_future)) as Box<GpuFuture>
    }

    pub fn get_texture(&self, name: ResourceID) -> std::sync::Arc<vulkano::image::immutable::ImmutableImage<vulkano::format::R8G8B8A8Srgb>> {
        self.textures[&name].clone()
    }
}