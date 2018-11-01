// #![windows_subsystem = "windows"]

extern crate engine;

fn main() {
    cute_log::init().expect("failed to initialize log!");

    engine::run();
}
