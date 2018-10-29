extern crate cgmath;
extern crate fnv;
extern crate gl;
extern crate glutin;
extern crate image;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate num;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate strum;
#[macro_use]
extern crate strum_macros;

mod renderer;
mod resource;


pub fn run() {
    info!("Creating window...");
    let mut window_manager = renderer::OpenGLManager::new();

    info!("Initializing resource manager...");
    let mut resource_manager = resource::ResourceManager::new();

    info!("Initializing render manager...");
    let mut render_manager = renderer::RenderManager::new();

    info!("Loading resources...");
    resource_manager.initialize();

    window_manager.run(|gl_window| {
        render_manager.main_loop(gl_window, &resource_manager);
    });

    resource_manager.shutdown();
}