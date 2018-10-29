use cgmath::Matrix;
use gl::types::*;
use std::ffi::CString;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Shader {
    pub id: GLuint,
}

#[derive(Clone, Copy, Display, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum CompilationType {
    #[strum(serialize="PROGRAM")]
    Program,
    #[strum(serialize="VERTEX")]
    Vertex,
    #[strum(serialize="FRAGMENT")]
    Fragment,
    #[strum(serialize="GEOMETRY")]
    Geometry
}

impl Shader {
    pub fn new() -> Self {
        Shader {
            id: 0,
        }
    }

    pub fn use_shader(self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn compile(&mut self, vertex_source: &str, fragment_source: &str, geometry_source: Option<&str>) {
        unsafe {
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let c_str_vert = CString::new(vertex_source.as_bytes()).unwrap();
            gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);
            check_compile_errors(vertex_shader, CompilationType::Vertex);

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_str_frag = CString::new(fragment_source.as_bytes()).unwrap();
            gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), std::ptr::null());
            gl::CompileShader(fragment_shader);
            check_compile_errors(fragment_shader, CompilationType::Fragment);

            let geometry_shader = if let Some(geo_source) = geometry_source {
                let geometry_shader = gl::CreateShader(gl::GEOMETRY_SHADER);
                let c_str_geo = CString::new(geo_source.as_bytes()).unwrap();
                gl::ShaderSource(geometry_shader, 1, &c_str_geo.as_ptr(), std::ptr::null());
                gl::CompileShader(geometry_shader);
                check_compile_errors(geometry_shader, CompilationType::Geometry);

                Some(geometry_shader)
            } else {
                None
            };

            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            if let Some(geo_shader) = geometry_shader {
                gl::AttachShader(shader_program, geo_shader);
            }
            gl::LinkProgram(shader_program);
            check_compile_errors(shader_program, CompilationType::Program);

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            if let Some(geo_shader) = geometry_shader {
                gl::DeleteShader(geo_shader);
            }

            self.id = shader_program;
        }
    }

    pub fn set_float(&self, name: &str, value: f32, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        unsafe {
            let c_str_name = CString::new(name.as_bytes()).unwrap();
            gl::Uniform1f(gl::GetUniformLocation(self.id, c_str_name.as_ptr()), value);
        }
    }

    pub fn set_integer(&self, name: &str, value: i32, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        unsafe {
            let c_str_name = CString::new(name.as_bytes()).unwrap();
            gl::Uniform1i(gl::GetUniformLocation(self.id, c_str_name.as_ptr()), value);
        }
    }

    pub fn set_vector2f(&self, name: &str, x: f32, y: f32, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        unsafe {
            let c_str_name = CString::new(name.as_bytes()).unwrap();
            gl::Uniform2f(gl::GetUniformLocation(self.id, c_str_name.as_ptr()), x, y);
        }
    }

    pub fn set_vector2(&self, name: &str, value: cgmath::Vector2<f32>, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        unsafe {
            let c_str_name = CString::new(name.as_bytes()).unwrap();
            gl::Uniform2f(gl::GetUniformLocation(self.id, c_str_name.as_ptr()), value.x, value.y);
        }
    }

    pub fn set_vector3f(&self, name: &str, x: f32, y: f32, z: f32, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        unsafe {
            let c_str_name = CString::new(name.as_bytes()).unwrap();
            gl::Uniform3f(gl::GetUniformLocation(self.id, c_str_name.as_ptr()), x, y, z);
        }
    }

    pub fn set_vector3(&self, name: &str, value: cgmath::Vector3<f32>, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        unsafe {
            let c_str_name = CString::new(name.as_bytes()).unwrap();
            gl::Uniform3f(gl::GetUniformLocation(self.id, c_str_name.as_ptr()), value.x, value.y, value.z);
        }
    }

    pub fn set_vector4f(&self, name: &str, x: f32, y: f32, z: f32, w: f32,  use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        unsafe {
            let c_str_name = CString::new(name.as_bytes()).unwrap();
            gl::Uniform4f(gl::GetUniformLocation(self.id, c_str_name.as_ptr()), x, y, z, w);
        }
    }

    pub fn set_vector4(&self, name: &str, value: cgmath::Vector4<f32>, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        unsafe {
            let c_str_name = CString::new(name.as_bytes()).unwrap();
            gl::Uniform4f(gl::GetUniformLocation(self.id, c_str_name.as_ptr()), value.x, value.y, value.z, value.w);
        }
    }

    pub fn set_matrix4(&self, name: &str, value: cgmath::Matrix4<f32>, use_shader: bool) {
        if use_shader {
            self.use_shader();
        }
        unsafe {
            let c_str_name = CString::new(name.as_bytes()).unwrap();
            gl::UniformMatrix4fv(gl::GetUniformLocation(self.id, c_str_name.as_ptr()), 1, gl::FALSE, value.as_ptr());
        }
    }
}

fn check_compile_errors(object: GLuint, obj_type: CompilationType) {
        let mut success = GLint::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);
        unsafe {
            info_log.set_len(512 - 1);
            if obj_type != CompilationType::Program {
                gl::GetShaderiv(object, gl::COMPILE_STATUS, &mut success);
                if success != GLint::from(gl::TRUE) {
                    gl::GetShaderInfoLog(object, 512, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
                    println!("ERROR::SHADER::{}::COMPILATION_FAILED\n{}", obj_type, std::str::from_utf8(&info_log).unwrap());
                }
            } else {
                gl::GetProgramiv(object, gl::LINK_STATUS, &mut success);
                if success != GLint::from(gl::TRUE) {
                    gl::GetProgramInfoLog(object, 512, std::ptr::null_mut(), info_log.as_mut_ptr() as *mut GLchar);
                    println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", std::str::from_utf8(&info_log).unwrap());
                }
            }
        }
    }