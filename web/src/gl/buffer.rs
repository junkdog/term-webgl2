use std::slice;
use crate::gl::GL;

/// Uploads a single struct to a WebGL buffer as raw bytes.
///
/// # Parameters
/// * `gl` - WebGL context
/// * `data` - Reference to struct to upload
/// * `target` - Buffer target (e.g., GL::ARRAY_BUFFER)
/// * `usage` - Usage hint (e.g., GL::STATIC_DRAW)
///
/// # Safety
/// Assumes the struct is aligned and has a memory layout compatible with WebGL.
/// No padding/alignment checks are performed.
pub(crate) fn buffer_upload_struct<T>(
    gl: &GL,
    target: u32,
    data: &T,
    usage: u32,
) {
    unsafe {
        let data_ptr = data as *const T as *const u8;
        let size = size_of::<T>();
        let view = js_sys::Uint8Array::view(slice::from_raw_parts(data_ptr, size));
        gl.buffer_data_with_array_buffer_view(target, &view, usage);
    }
}

/// Uploads an array of elements to a WebGL buffer as raw bytes.
///
/// # Parameters
/// * `gl` - WebGL context
/// * `data` - Reference to array to upload
/// * `target` - Buffer target (e.g., GL::ARRAY_BUFFER)
/// * `usage` - Usage hint (e.g., GL::STATIC_DRAW)
///
/// # Safety
/// Assumes the elements are aligned and has a memory layout compatible with WebGL.
/// No padding/alignment checks are performed.
pub(crate) fn buffer_upload_array<T>(
    gl: &GL,
    target: u32,
    data: &[T],
    usage: u32,
) {
    unsafe {
        let data_ptr = data.as_ptr() as *const u8;
        let size = data.len() * size_of::<T>();
        let view = js_sys::Uint8Array::view(slice::from_raw_parts(data_ptr, size));
        gl.buffer_data_with_array_buffer_view(target, &view, usage);
    }
}
