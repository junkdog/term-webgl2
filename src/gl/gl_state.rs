use crate::gl::GL;

/// Manages simple WebGL state to reduce redundant state changes
pub struct GlState {
    // Capability flags
    blend_enabled: bool,
    depth_test_enabled: bool,
    cull_face_enabled: bool,

    // Viewport dimensions
    viewport: [i32; 4], // [x, y, width, height]

    // Clear color
    clear_color: [f32; 4],

    // Blend function state
    blend_func: (u32, u32), // (src_factor, dst_factor)

    // Active texture unit
    active_texture_unit: u32,

    // Enabled vertex attribute arrays
    enabled_vertex_attribs: Vec<bool>,
}

impl GlState {
    /// Create a new GLState object with WebGL defaults
    pub fn new(gl: &GL) -> Self {
        // Get max vertex attributes
        let max_vertex_attribs = gl.get_parameter(GL::MAX_VERTEX_ATTRIBS)
            .unwrap()
            .as_f64()
            .unwrap() as usize;

        Self {
            blend_enabled: false,
            depth_test_enabled: false,
            cull_face_enabled: false,
            viewport: [0, 0, 0, 0],
            clear_color: [0.0, 0.0, 0.0, 0.0],
            blend_func: (GL::ONE, GL::ZERO), // Default blend function
            active_texture_unit: GL::TEXTURE0,
            enabled_vertex_attribs: vec![false; max_vertex_attribs],
        }
    }

    /// Enable or disable blending
    pub fn enable_blend(&mut self, gl: &GL, enable: bool) -> &mut Self {
        if self.blend_enabled != enable {
            self.capability(gl, GL::BLEND, enable);
            self.blend_enabled = enable;
        }
        self
    }

    /// Set blend function
    pub fn blend_func(&mut self, gl: &GL, sfactor: u32, dfactor: u32) -> &mut Self {
        if self.blend_func != (sfactor, dfactor) {
            gl.blend_func(sfactor, dfactor);
            self.blend_func = (sfactor, dfactor);
        }
        self
    }

    /// Enable or disable depth testing
    pub fn enable_depth_test(&mut self, gl: &GL, enable: bool) -> &mut Self {
        if self.depth_test_enabled != enable {
            self.capability(gl, GL::DEPTH_TEST, enable);
            self.depth_test_enabled = enable;
        }
        self
    }

    /// Enable or disable face culling
    pub fn enable_cull_face(&mut self, gl: &GL, enable: bool) -> &mut Self {
        if self.cull_face_enabled != enable {
            self.capability(gl, GL::CULL_FACE, enable);
            self.cull_face_enabled = enable;
        }
        self
    }

    /// Set viewport dimensions
    pub fn viewport(&mut self, gl: &GL, x: i32, y: i32, width: i32, height: i32) -> &mut Self {
        let new_viewport = [x, y, width, height];
        if self.viewport != new_viewport {
            gl.viewport(x, y, width, height);
            self.viewport = new_viewport;
        }
        self
    }

    /// Set clear color
    pub fn clear_color(&mut self, gl: &GL, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        let new_color = [r, g, b, a];
        if self.clear_color != new_color {
            gl.clear_color(r, g, b, a);
            self.clear_color = new_color;
        }
        self
    }

    /// Set active texture unit
    pub fn active_texture(&mut self, gl: &GL, texture_unit: u32) -> &mut Self {
        if self.active_texture_unit != texture_unit {
            gl.active_texture(texture_unit);
            self.active_texture_unit = texture_unit;
        }
        self
    }

    /// Enable or disable a vertex attribute array
    pub fn vertex_attrib_array(&mut self, gl: &GL, index: u32, enable: bool) -> &mut Self {
        let idx = index as usize;
        if idx < self.enabled_vertex_attribs.len() && self.enabled_vertex_attribs[idx] != enable {
            if enable {
                gl.enable_vertex_attrib_array(index);
            } else {
                gl.disable_vertex_attrib_array(index);
            }
            self.enabled_vertex_attribs[idx] = enable;
        }
        self
    }

    /// Reset all tracked state to WebGL defaults
    pub fn reset(&mut self, gl: &GL) {
        // Reset capabilities
        if self.blend_enabled {
            gl.disable(GL::BLEND);
            self.blend_enabled = false;
        }

        if self.depth_test_enabled {
            gl.disable(GL::DEPTH_TEST);
            self.depth_test_enabled = false;
        }

        if self.cull_face_enabled {
            gl.disable(GL::CULL_FACE);
            self.cull_face_enabled = false;
        }

        // Reset blend function
        if self.blend_func != (GL::ONE, GL::ZERO) {
            gl.blend_func(GL::ONE, GL::ZERO);
            self.blend_func = (GL::ONE, GL::ZERO);
        }

        // Reset texture unit
        if self.active_texture_unit != GL::TEXTURE0 {
            gl.active_texture(GL::TEXTURE0);
            self.active_texture_unit = GL::TEXTURE0;
        }

        // Reset vertex attributes
        for (idx, enabled) in self.enabled_vertex_attribs.iter_mut().enumerate() {
            if *enabled {
                gl.disable_vertex_attrib_array(idx as u32);
                *enabled = false;
            }
        }

        // Note: We don't reset viewport or clear_color as these are typically
        // set based on canvas dimensions or application needs
    }
    
    fn capability(&self, gl: &GL, capability: u32, enable: bool) {
        if enable {
            gl.enable(capability);
        } else {
            gl.disable(capability);
        }
    }
}


pub struct BoundGlState<'a> {
    gl: &'a web_sys::WebGl2RenderingContext,
    state: &'a mut GlState,
}

impl<'a> BoundGlState<'a> {
    pub(crate) fn new(
        gl: &'a web_sys::WebGl2RenderingContext,
        state: &'a mut GlState
    ) -> Self {
        Self { gl, state }
    }
    
    /// Enable or disable blending
    pub fn enable_blend(&mut self, enable: bool) -> &mut Self {
        self.state.enable_blend(self.gl, enable);
        self
    }

    /// Set blend function
    pub fn blend_func(&mut self, sfactor: u32, dfactor: u32) -> &mut Self {
        self.state.blend_func(self.gl, sfactor, dfactor);
        self
    }

    /// Enable or disable depth testing
    pub fn enable_depth_test(&mut self, enable: bool) -> &mut Self {
        self.state.enable_depth_test(self.gl, enable);
        self
    }

    /// Enable or disable face culling
    pub fn enable_cull_face(&mut self, enable: bool) -> &mut Self {
        self.state.enable_cull_face(self.gl, enable);
        self
    }

    /// Set viewport dimensions
    pub fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32) -> &mut Self {
        self.state.viewport(self.gl, x, y, width, height);
        self
    }

    /// Set clear color
    pub fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) -> &mut Self {
        self.state.clear_color(self.gl, r, g, b, a);
        self
    }

    /// Set active texture unit
    pub fn active_texture(&mut self, texture_unit: u32) -> &mut Self {
        self.state.active_texture(self.gl, texture_unit);
        self
    }

    /// Enable or disable a vertex attribute array
    pub fn vertex_attrib_array(&mut self, index: u32, enable: bool) -> &mut Self {
        self.state.vertex_attrib_array(self.gl, index, enable);
        self
    }

    /// Reset all tracked state to WebGL defaults
    pub fn reset(&mut self) {
        self.state.reset(self.gl);
    }

    fn capability(&self, capability: u32, enable: bool) {
        self.state.capability(self.gl, capability, enable);
    }
}