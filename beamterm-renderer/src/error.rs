/// Error categories.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to initialize WebGL context or retrieve DOM elements.
    #[error("Initialization error: {0}")]
    Initialization(String),

    /// Shader compilation, linking, or program creation errors.
    #[error("Shader error: {0}")]
    Shader(String),

    /// WebGL resource creation or management errors.
    #[error("Resource error: {0}")]
    Resource(String),

    /// External data loading or parsing errors.
    #[error("Data error: {0}")]
    Data(String),

    /// Event listener errors, related to mouse input handling.
    #[error("Event listener error: {0}")]
    Callback(String),
}

impl Error {
    // Helper constructors for common error scenarios

    // Initialization errors
    pub fn window_not_found() -> Self {
        Self::Initialization("Unable to retrieve window".to_string())
    }

    pub fn document_not_found() -> Self {
        Self::Initialization("Unable to retrieve document".to_string())
    }

    pub fn canvas_not_found() -> Self {
        Self::Initialization("Unable to retrieve canvas".to_string())
    }

    pub fn webgl_context_failed() -> Self {
        Self::Initialization("Failed to retrieve WebGL2 rendering context".to_string())
    }

    pub fn canvas_context_failed() -> Self {
        Self::Initialization("Failed to retrieve canvas rendering context".to_string())
    }

    // Shader errors
    pub fn shader_creation_failed(detail: &str) -> Self {
        Self::Shader(format!("Shader creation failed: {detail}"))
    }

    pub fn shader_program_creation_failed() -> Self {
        Self::Shader("Shader program creation failed".to_string())
    }

    pub fn shader_link_failed(log: String) -> Self {
        Self::Shader(format!("Shader linking failed: {log}"))
    }

    // Resource errors
    pub fn buffer_creation_failed(buffer_type: &str) -> Self {
        Self::Resource(format!("Failed to create {buffer_type} buffer"))
    }

    pub fn vertex_array_creation_failed() -> Self {
        Self::Resource("Failed to create vertex array object".to_string())
    }

    pub fn texture_creation_failed() -> Self {
        Self::Resource("Failed to create texture".to_string())
    }

    pub fn uniform_location_failed(name: &str) -> Self {
        Self::Resource(format!("Failed to get uniform location: {name}"))
    }

    pub fn webgl_error(message: String) -> Self {
        Self::Resource(format!("WebGL error: {message}"))
    }

    pub fn element_creation_failed(element_type: &str) -> Self {
        Self::Resource(format!("Failed to create element: {element_type}"))
    }

    // Data errors
    pub fn image_load_failed(path: &str) -> Self {
        Self::Data(format!("Failed to load image: {path}"))
    }

    pub fn deserialization_failed(message: String) -> Self {
        Self::Data(format!("Failed to deserialize: {message}"))
    }
}
