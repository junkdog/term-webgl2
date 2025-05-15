mod shader_program;
mod vertex;
mod renderer;
mod texture;
mod gl_state;

pub(crate) use shader_program::*;
pub(crate) use vertex::*;
pub(crate) use renderer::*;
pub(crate) use texture::*;

pub(crate) type GL = web_sys::WebGl2RenderingContext;