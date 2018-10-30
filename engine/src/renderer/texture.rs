use gl::types::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Texture {
    pub id: GLuint,
}

impl Texture {
    pub fn new() -> Self {
        Texture {
            id: 0
        }
    }

    pub fn from_source_rgba(&mut self, image_width: u32, image_height: u32, source: &[u8]) {
        unsafe {
            gl::GenTextures(1, &mut self.id);
            gl::BindTexture(gl::TEXTURE_2D, self.id);

            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as _, image_width as _, image_height as _, 0, gl::RGBA, gl::UNSIGNED_BYTE, source.as_ptr() as _);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR_MIPMAP_LINEAR as _);
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }
    }
}