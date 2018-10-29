pub mod shader;
pub mod texture;

use gl::types::*;
use glutin::dpi::*;
use glutin::GlContext;

use std::mem;
use std::os::raw::c_void;

use cgmath::prelude::*;

const WIDTH: f32 = 1280.0;
const HEIGHT: f32 = 720.0;
const FOV: f32 = 90.0;

pub struct OpenGLManager {
    events_loop: glutin::EventsLoop,
    pub gl_window: glutin::GlWindow,
}

impl OpenGLManager {
    pub fn new() -> Self {
        let events_loop = glutin::EventsLoop::new();

        let window = glutin::WindowBuilder::new()
            .with_title("Rusty RPG")
            .with_dimensions(LogicalSize::new(WIDTH.into(), HEIGHT.into()));
        let context = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(8);
        
        let gl_window = glutin::GlWindow::new(window, context, &events_loop)
            .expect("failed to create window!");
        
        unsafe {
            gl_window.make_current().expect("failed to make current");
        }
        
        gl::load_with(|s| gl_window.get_proc_address(s) as *const _);

        OpenGLManager {
            events_loop,
            gl_window,
        }
    }

    pub fn run<F>(&mut self, mut main_loop: F)
    where F: FnMut(&mut glutin::GlWindow) -> () {
        let mut running = true;
        while running {
            {
                let gl_window = &self.gl_window;

                self.events_loop.poll_events(|event| {
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
            }

            main_loop(&mut self.gl_window);
        }
    }
}

pub struct RenderManager {
    vertex_buffer: GLuint,
}

impl RenderManager {
    pub fn new() -> Self {
        // Create VAO
        let mut vertex_array_id: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array_id as *mut GLuint);
            gl::BindVertexArray(vertex_array_id);
        }

        const G_VERTEX_BUFFER_DATA: [GLfloat; 9] = [
            -1.0, -1.0, 0.0,
            1.0, -1.0, 0.0,
            0.0, 1.0, 0.0
        ];

        let mut vertex_buffer: GLuint = 0;

        unsafe {
            gl::GenBuffers(1, &mut vertex_buffer as *mut GLuint);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                (G_VERTEX_BUFFER_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &G_VERTEX_BUFFER_DATA[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW);
        }

        unsafe {
            gl::ClearColor(0.0, 1.0, 0.0, 1.0);
        }

        RenderManager {
            vertex_buffer,
        }
    }

    pub fn main_loop(&mut self, gl_window: &mut glutin::GlWindow, resource_manager: &crate::resource::ResourceManager) {
        // Create transformations
        let projection = cgmath::perspective(cgmath::Deg(FOV), WIDTH / HEIGHT, 0.1, 100.0);
        // let projection = cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 0.0, 100.0);
        let view = cgmath::Matrix4::look_at(cgmath::Point3::new(4., 3., 3.),
                                            cgmath::Point3::new(0., 0., 0.),
                                            cgmath::vec3(0., 1., 0.));
        let model: cgmath::Matrix4<f32> = cgmath::Matrix4::identity();

        let mvp = projection * view * model;

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            let shader_id = crate::resource::ResourceID::from("sprite_shader");
            let shader_program = resource_manager.get_shader(shader_id);
            shader_program.use_shader();
            shader_program.set_matrix4("MVP", mvp, false);

            gl::EnableVertexAttribArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                0,
                std::ptr::null()
            );

            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::DisableVertexAttribArray(0);
        }

        gl_window.swap_buffers().unwrap();
    }
}