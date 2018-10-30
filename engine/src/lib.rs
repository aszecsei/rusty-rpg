#![feature(test)]

mod renderer;
mod resource;
mod time;

use log::info;

pub fn run() {
    info!("Creating window...");
    let mut window_manager = renderer::OpenGLManager::new();

    info!("Initializing resource manager...");
    let mut resource_manager = resource::ResourceManager::new();

    info!("Initializing render manager...");
    let mut render_manager = renderer::RenderManager::new();

    info!("Loading resources...");
    resource_manager.initialize();

    info!("Starting timer...");
    let mut time_manager = time::TimeManager::new();

    window_manager.run(|gl_window| {
        let delta_time = time_manager.tick();
        let delta_time_float = (delta_time.as_secs() as f64) + (f64::from(delta_time.subsec_nanos()) * 1e-9);
        
        render_manager.main_loop(gl_window, &resource_manager, delta_time_float);
    });

    resource_manager.shutdown();
}