extern crate gl;
extern crate glutin;

use gl::types::*;
use glutin::dpi::*;
use glutin::GlContext;

pub fn run() {
    let mut events_loop = glutin::EventsLoop::new();

    // enumerate monitors
    let monitor = events_loop.get_primary_monitor();
    println!("Creating window on {:?}", monitor.get_name());

    let window = glutin::WindowBuilder::new()
        .with_title("Rusty RPG")
        .with_dimensions(LogicalSize::new(1280.0, 720.0));
    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    
    let gl_window = glutin::GlWindow::new(window, context, &events_loop)
        .expect("failed to create window!");
    
    unsafe {
        gl_window.make_current().expect("failed to make current");
    }
    
    gl::load_with(|s| gl_window.get_proc_address(s) as *const _);

    unsafe {
        gl::ClearColor(0.0, 1.0, 0.0, 1.0);
    }

    let mut running = true;
    while running {
        events_loop.poll_events(|event| {
            if let glutin::Event::WindowEvent{ event, .. } = event {
                match event {
                    glutin::WindowEvent::CloseRequested => running = false,
                    glutin::WindowEvent::Resized(logical_size) => {
                        let dpi_factor = gl_window.get_hidpi_factor();
                        gl_window.resize(logical_size.to_physical(dpi_factor));
                    },
                    _ => ()
                }
            }
        });

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        gl_window.swap_buffers().unwrap();
    }
}