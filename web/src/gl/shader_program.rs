use crate::error::Error;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};
use crate::gl::GL;
use crate::mat4::Mat4;

pub(crate) struct ShaderProgram {
    pub(crate) program: WebGlProgram
}

impl ShaderProgram {
    pub(super) fn create(
        gl: &WebGl2RenderingContext,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Self, Error> {
        let program = gl.create_program()
            .ok_or(Error::ShaderProgramCreationError)?;
    
        // compile shaders
        let vertex_shader = compile_shader(gl, ShaderType::Vertex, vertex_source)?;
        let fragment_shader = compile_shader(gl, ShaderType::Fragment, fragment_source)?;
        
        // attach shaders and link program
        gl.attach_shader(&program, &vertex_shader);
        gl.attach_shader(&program, &fragment_shader);
        gl.link_program(&program);
        check_link_status(gl, &program)?;
        
        // delete shaders (no longer needed after linking)
        gl.delete_shader(Some(&vertex_shader));
        gl.delete_shader(Some(&fragment_shader));
    
        Ok(ShaderProgram {
            program
        })
    }

    pub fn set_uniform_mat4(
        &self,
        gl: &WebGl2RenderingContext,
        name: &'static str,
        matrix: &Mat4
    ) -> Result<(), Error> {
        self.use_program(gl);
        let location = gl.get_uniform_location(&self.program, name)
            .ok_or(Error::UnableToRetrieveUniformLocation(name))?;

        gl.uniform_matrix4fv_with_f32_array(
            Some(&location),
            false,  // don't transpose
            matrix.as_slice()
        );

        Ok(())
    }

    pub(crate) fn set_uniform_vec2(
        &self, gl: &GL,
        name: &'static str,
        x: f32,
        y: f32
    )  -> Result<(), Error> {
        self.use_program(gl);
        let location = gl.get_uniform_location(&self.program, name)
            .ok_or(Error::UnableToRetrieveUniformLocation(name))?;

        gl.uniform2f(Some(&location), x, y);

        Ok(())
    }

    /// Use the shader program.
    pub(crate) fn use_program(&self, gl: &WebGl2RenderingContext) {
        gl.use_program(Some(&self.program));
    }
}

fn compile_shader(
    gl: &WebGl2RenderingContext,
    shader_type: ShaderType,
    source: &str,
) -> Result<WebGlShader, Error> {
    let shader = gl.create_shader(shader_type.into())
        .ok_or(Error::ShaderCreationError("failed creating shader"))?;

    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    Ok(shader)
}

fn check_link_status(
    gl: &WebGl2RenderingContext,
    program: &WebGlProgram,
) -> Result<(), Error> {
    let status = gl.get_program_parameter(program, WebGl2RenderingContext::LINK_STATUS);
    if !status.as_bool().unwrap() {
        gl.get_program_info_log(program)
            .map(Error::ShaderLinkError)
            .ok_or(Error::ShaderProgramCreationError)?;
    }

    Ok(())
}

/// Enum representing the type of shader.
enum ShaderType {
    Vertex,
    Fragment,
}

impl Into<u32> for ShaderType {
    fn into(self) -> u32 {
        use ShaderType::*;

        match self {
            Vertex   => WebGl2RenderingContext::VERTEX_SHADER,
            Fragment => WebGl2RenderingContext::FRAGMENT_SHADER,
        }
    }
}