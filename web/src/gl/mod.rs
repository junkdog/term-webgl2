mod program;
mod vertex;
mod renderer;
mod texture;
mod context;

pub(crate) use program::*;
pub(crate) use vertex::*;
pub(crate) use renderer::*;
pub(crate) use texture::*;

pub(crate) type GL = web_sys::WebGl2RenderingContext;