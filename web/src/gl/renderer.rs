use crate::error::Error;
use crate::gl::context::{BoundGlState, GlState};
use crate::gl::GL;
use crate::js;
use crate::mat4::Mat4;

pub(crate) struct RenderContext<'a> {
    pub(crate) gl: &'a web_sys::WebGl2RenderingContext,
    pub(crate) state: &'a mut GlState,
    pub(crate) projection: Mat4
}

pub(crate) struct Renderer {
    gl: web_sys::WebGl2RenderingContext,
    canvas: web_sys::HtmlCanvasElement,
    state: GlState,
    projection: Mat4
}

impl Renderer {
    pub fn create(canvas_id: &str) -> Result<Self, Error> {
        let canvas = js::get_canvas_by_id(canvas_id)?;
        let (width, height) = (canvas.width(), canvas.height());

        // initialize WebGL context
        let gl = js::get_webgl2_context(&canvas)?;
        let state = GlState::new(&gl);
        let projection = Mat4::orthographic_from_size(width as f32, height as f32);

        let mut renderer = Self { gl, projection, canvas, state };
        renderer.resize(width as _, height as _);
        Ok(renderer)
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

    pub fn begin_frame(&mut self) {
        self.clear(0.0, 0.0, 0.0);
    }

    pub fn render(&mut self, drawable: &impl Drawable) {
        let mut context = RenderContext {
            gl: &self.gl,
            state: &mut self.state,
            projection: self.projection.clone(),
        };

        drawable.prepare(&mut context);
        drawable.draw(&mut context);
        drawable.cleanup(&mut context);
    }

    pub fn end_frame(&mut self) {
        // swap buffers (todo)
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

    pub fn canvas_size(&self) -> (i32, i32) {
        (self.canvas.width() as i32, self.canvas.height() as i32)
    }

    pub fn canvas_height(&self) -> i32 {
        self.canvas.height() as i32
    }
}

pub(crate) trait Drawable {
    fn prepare(&self, context: &mut RenderContext);
    fn draw(&self, context: &mut RenderContext);
    fn cleanup(&self, context: &mut RenderContext);
}
