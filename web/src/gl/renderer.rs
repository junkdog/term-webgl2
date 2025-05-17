use std::slice;
use crate::error::Error;
use crate::gl::gl_state::{BoundGlState, GlState};
use crate::gl::{ShaderProgram, GL};
use js_sys::wasm_bindgen::JsCast;
use crate::js;

pub(crate) struct Renderer {
    gl: web_sys::WebGl2RenderingContext,
    canvas: web_sys::HtmlCanvasElement,
    state: GlState,
}

impl Renderer {
    pub fn create(canvas_id: &str) -> Result<Self, Error> {
        let document = js::document()?;

        let canvas = document.query_selector(canvas_id)
            .map_err(|_| Error::UnableToRetrieveCanvas)?
            .ok_or(Error::UnableToRetrieveCanvas)?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| Error::UnableToRetrieveCanvas)?;
        
        let (width, height) = (canvas.width(), canvas.height());

        // initialize WebGL context
        let gl = js::get_webgl2_context(&canvas)?;

        let state = GlState::new(&gl);
        
        let mut renderer = Self { gl, canvas, state };
        renderer.resize(width as _, height as _);
        Ok(renderer)
    }

    pub fn create_shader_program(&self, vert_src: &str, frag_src: &str) -> Result<ShaderProgram, Error> {
        ShaderProgram::create(&self.gl, vert_src, frag_src)
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        self.state.viewport(&self.gl, 0, 0, width, height);
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32) {
        self.state.clear_color(&self.gl, r, g, b, 1.0);
        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);
    }

    pub fn draw(&self, shader: &ShaderProgram, vertex_array: &impl Drawable) {
        shader.use_program(&self.gl);
        vertex_array.bind(&self.gl);
        vertex_array.draw(&self.gl);
        vertex_array.unbind(&self.gl);
    }

    // Accessor methods
    pub fn gl(&self) -> &GL {
        &self.gl
    }
    
    pub fn state(&mut self) -> BoundGlState {
        BoundGlState::new(&self.gl, &mut self.state)
    }

    pub fn canvas_width(&self) -> i32 {
        self.canvas.width() as i32
    }

    pub fn canvas_height(&self) -> i32 {
        self.canvas.height() as i32
    }
}

pub(crate) trait Drawable {
    fn bind(&self, gl: &web_sys::WebGl2RenderingContext);
    fn draw(&self, gl: &web_sys::WebGl2RenderingContext);
    fn unbind(&self, gl: &web_sys::WebGl2RenderingContext);
}

#[repr(C, align(4))]
pub(crate) struct InstanceData {
    pub position: [u16; 2],
    pub depth: f32,
    pub fg: u32,
    pub bg: u32,
}

impl InstanceData {
    const STRIDE: i32 = size_of::<Self>() as i32;

    pub(crate) const POS_ATTRIB: u32 = 2;
    pub(crate) const DEPTH_ATTRIB: u32 = 3;
    pub(crate) const FG_ATTRIB: u32 = 4;
    pub(crate) const BG_ATTRIB: u32 = 5;

    pub(crate) fn new(xy: (u16, u16), depth: u16, fg: u32, bg: u32) -> Self {
        Self { position: [xy.0, xy.1], depth: depth as f32, fg, bg }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitmap_font::BitmapFontMetadata;
    use crate::gl::texture::Texture;

    #[test]
    fn test_renderer() {
        println!("instanve size: {}", InstanceData::STRIDE);
    }

}