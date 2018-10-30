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

// static FONT: &[u8] = include_bytes!("../../../resources/fonts/FRABK.ttf");

pub struct OpenGLManager {
    events_loop: glutin::EventsLoop,
    pub gl_window: glutin::GlWindow,
}

impl OpenGLManager {
    pub fn new() -> Self {
        let events_loop = glutin::EventsLoop::new();

        let window = glutin::WindowBuilder::new()
            .with_title("Rusty RPG")
            .with_resizable(false)
            .with_dimensions(LogicalSize::new(WIDTH.into(), HEIGHT.into()));
        let context = glutin::ContextBuilder::new()
            .with_srgb(true)
            // .with_vsync(true)
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
        let mut loop_helper = spin_sleep::LoopHelper::builder()
            .report_interval_s(0.5) // report every half a second
            .build_with_target_rate(250.0);
        let mut running = true;
        while running {
            loop_helper.loop_start();

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
                            glutin::WindowEvent::KeyboardInput {
                                input:
                                    glutin::KeyboardInput {
                                        state: glutin::ElementState::Pressed,
                                        virtual_keycode: Some(keypress),
                                        ..
                                    },
                                ..
                            } => match keypress {
                                glutin::VirtualKeyCode::Escape => running = false,
                                _ => (),
                            },
                            _ => ()
                        }
                    }
                });
            }

            main_loop(&mut self.gl_window);

            if let Some(rate) = loop_helper.report_rate() {
                self.gl_window.set_title(&format!("{} {:.0} FPS", "Rusty RPG", rate));
            }

            loop_helper.loop_sleep();
        }
    }
}

pub struct RenderManager {
    vertex_buffer: GLuint,
}

impl RenderManager {
    pub fn new() -> Self {
        // Create VAO
        const VERTICES: [GLfloat; 32] = [
            // positions        // colors       // texture coords
            0.5, 0.5, 0.0,      1.0, 0.0, 0.0,  1.0, 1.0,   // top right
            0.5, -0.5, 0.0,     0.0, 1.0, 0.0,  1.0, 0.0,   // bottom right
            -0.5, -0.5, 0.0,    0.0, 0.0, 1.0,  0.0, 0.0,   // bottom left
            -0.5, 0.5, 0.0,     1.0, 1.0, 0.0,  0.0, 1.0,
        ];
        const INDICES: [GLuint; 6] = [
            0, 1, 3, // first triangle
            1, 2, 3, // second triangle
        ];

        // bind vertex array object
        let mut vertex_array_id: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vertex_array_id);
            gl::BindVertexArray(vertex_array_id);
        }

        // copy vertex data into a vertex buffer
        let mut vertex_buffer: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut vertex_buffer);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertex_buffer);
            gl::BufferData(gl::ARRAY_BUFFER,
                (VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &VERTICES[0] as *const f32 as *const c_void,
                gl::STATIC_DRAW);
        }

        // copy index array into an element buffer
        let mut element_buffer: GLuint = 0;
        unsafe {
            gl::GenBuffers(1, &mut element_buffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, element_buffer);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER,
                (INDICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                &INDICES[0] as *const u32 as *const c_void,
                gl::STATIC_DRAW);
        }

        unsafe {
            // Enable alpha blending
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::Enable(gl::FRAMEBUFFER_SRGB);

            gl::ClearColor(0.0, 1.0, 0.0, 1.0);
        }

        RenderManager {
            vertex_buffer,
        }
    }

    fn draw(&self, resource_manager: &crate::resource::ResourceManager) {
        // Create transformations
        let projection = cgmath::perspective(cgmath::Deg(FOV), WIDTH / HEIGHT, 0.1, 100.0);
        // let projection = cgmath::ortho(-10.0, 10.0, -10.0, 10.0, 0.0, 100.0);
        let view = cgmath::Matrix4::look_at(cgmath::Point3::new(0., 0., 4.),
                                            cgmath::Point3::new(0., 0., 0.),
                                            cgmath::vec3(0., 1., 0.));
        let model: cgmath::Matrix4<f32> = cgmath::Matrix4::identity();

        let mvp = projection * view * model;

        let shader_id = crate::resource::ResourceID::from("sprite_shader");
        let shader_program = resource_manager.get_shader(shader_id);
        shader_program.use_shader();
        shader_program.set_matrix4("MVP", mvp, false);

        let tex_id = crate::resource::ResourceID::from("awesome_face");
        let tex = resource_manager.get_texture(tex_id);

        unsafe {        
            // Positions
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (8 * mem::size_of::<GLfloat>()) as i32,
                std::ptr::null()
            );
            gl::EnableVertexAttribArray(0);

            // Colors
            gl::VertexAttribPointer(
                1,
                3,
                gl::FLOAT,
                gl::FALSE,
                (8 * mem::size_of::<GLfloat>()) as i32,
                (3 * mem::size_of::<GLfloat>()) as *const _,
            );
            gl::EnableVertexAttribArray(1);

            // Texture coords
            gl::VertexAttribPointer(
                2,
                2,
                gl::FLOAT,
                gl::FALSE,
                (8 * mem::size_of::<GLfloat>()) as i32,
                (6 * mem::size_of::<GLfloat>()) as *const _,
            );
            gl::EnableVertexAttribArray(2);

            gl::BindTexture(gl::TEXTURE_2D, tex.id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());
            
            gl::DisableVertexAttribArray(0);
            gl::DisableVertexAttribArray(1);
            gl::DisableVertexAttribArray(2);
        }
    }

    pub fn main_loop(&mut self, gl_window: &mut glutin::GlWindow, resource_manager: &crate::resource::ResourceManager, delta_time: f64) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        self.draw(resource_manager);

        gl_window.swap_buffers().unwrap();
    }
}