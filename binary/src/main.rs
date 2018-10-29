// #![windows_subsystem = "windows"]

extern crate engine;

extern crate cute_log;

fn main() {
    cute_log::init().expect("failed to initialize log!");
    
    engine::run();
}
