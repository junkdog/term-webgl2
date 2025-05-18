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

    #[error("Unable to retrieve CanvasRenderingContext2d")]
    FailedToRetrieveCanvasRenderingContext2d,
    
    /// WebGL error.
    #[error("WebGL error: {0}")]
    WebGlError(String),
    
    #[error("Failed to create buffer: {0}")]
    BufferCreationError(&'static str),
    
    #[error("Failed to create vertex array object")]
    VertexArrayCreationError,
    
    #[error("Failed to create textue")]
    TextureCreationError,
    
    #[error("Failed to create uniform location: {0}")]
    UnableToRetrieveUniformLocation(&'static str),
    
    #[error("Failed to load image from path: {0}")]
    ImageLoadError(&'static str),
    
    #[error("Failed to creeate element: {0}")]
    UnableToCreateElement(&'static str),
    
    #[error("Failed to deserialize JSON: {0}")]
    JsonDeserializationError(String),
}