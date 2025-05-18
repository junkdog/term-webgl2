mod font_atlas;
mod error;
mod mat4;
mod gl;

pub(crate) mod js;

pub use crate::font_atlas::FontAtlasConfig;
pub use crate::error::Error;
pub use crate::gl::{FontAtlas, Renderer, TerminalGrid};
pub use mat4::Mat4;