/// Custom error implementation.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Unable to create shader program.
    #[error("Shader program creation error")]
    ShaderProgramCreationError,

    /// Unable to create shader program.
    #[error("Shader creation error: {0}")]
    ShaderCreationError(&'static str),

    /// Unable to link shader program.
    #[error("Failed linking shader: {0}")]
    ShaderLinkError(String),
    
    /// Unable to retrieve window.
    #[error("Unable to retrieve window")]
    UnableToRetrieveWindow,

    /// Unable to retrieve document.
    #[error("Unable to retrieve document")]
    UnableToRetrieveDocument,

    /// Unable to retrieve body.
    #[error("Unable to retrieve body")]
    UnableToRetrieveBody,

    /// Unable to retrieve canvas.
    #[error("Unable to retrieve canvas")]
    UnableToRetrieveCanvas,
    
    /// Unable to retrieve WebGl2RenderingContext.
    #[error("Unable to retrieve WebGl2RenderingContext")]
    FailedToRetrieveWebGl2RenderingContext,
    
    /// WebGL error.
    #[error("WebGL error: {0}")]
    WebGlError(String),
    
    #[error("Failed to create buffer")]
    BufferCreationError,
    
    #[error("Failed to create vertex array object")]
    VertexArrayCreationError,
}