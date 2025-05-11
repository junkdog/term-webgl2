use image::GenericImageView;
use web_sys::console;
use crate::error::Error;
use crate::gl::GL;

pub struct Texture {
    gl_texture: web_sys::WebGlTexture,
    width: i32,
    height: i32,
}

impl Texture {
    pub fn from_image_data(
        gl: &web_sys::WebGl2RenderingContext,
        format: u32,
        image_data: &[u8],
    ) -> Result<Self, Error> {
        // load the image
        let img = image::load_from_memory(image_data)
            .map_err(|e| {
                console::error_1(&format!("Failed to load image: {:?}", e).into());
                Error::ImageLoadError("failed to load image data")
            })?;

        // convert the image to RGBA format
        let (width, height) = img.dimensions();
        console::log_1(&format!("Image dimensions: {}x{}", width, height).into());
        
        let rgba_image = img.to_rgba8();
        let raw_data = rgba_image.as_raw();

        // create the texture
        Self::new(gl, format, raw_data, width as i32, height as i32)
    }
    pub fn from_image(
        gl: &web_sys::WebGl2RenderingContext,
        format: u32,
        path: &'static str
    ) -> Result<Self, Error> {
        // load the image
        let img = image::open(path)
            .map_err(|_| Error::ImageLoadError(path))?;

        Self::from_image_data(gl, format, &img.to_rgba8().into_raw())
    }

    pub fn new(
        gl: &web_sys::WebGl2RenderingContext,
        format: u32,
        data: &[u8],
        width: i32,
        height: i32
    ) -> Result<Self, Error> {
        console::log_1(&format!("Creating texture with format: {}, width: {}, height: {}", format, width, height).into());
        console::log_1(&format!("Data length: {}", data.len()).into());

        // expected data length for error checking
        let expected_length = (width * height * 4) as usize; // 4 bytes per pixel for RGBA
        if data.len() != expected_length && format == GL::RGBA {
            console::warn_1(&format!("Data length mismatch: got {}, expected {}", data.len(), expected_length).into());
        }
        
        
        
        let gl_texture = gl.create_texture()
            .ok_or(Error::TextureCreationError)?;
        gl.bind_texture(GL::TEXTURE_2D, Some(&gl_texture));
        
        unsafe {
            let view = js_sys::Uint8Array::view(data);
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
                GL::TEXTURE_2D,
                0,
                format as i32,
                width,
                height,
                0,
                format,
                GL::UNSIGNED_BYTE,
                &view,
                0,
            ).map_err(|v| {
                console::error_2(&"Failed to upload texture data".into(), &v);
                Error::TextureCreationError
            })?;
        }

        gl.generate_mipmap(GL::TEXTURE_2D);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);

        Ok(Self { gl_texture, width, height })
    }

    pub fn bind(&self, gl: &web_sys::WebGl2RenderingContext, texture_unit: u32) {
        // bind texture and set uniform
        gl.active_texture(GL::TEXTURE0 + texture_unit);
        gl.bind_texture(GL::TEXTURE_2D, Some(&self.gl_texture));
    }

    pub fn delete(&self, gl: &web_sys::WebGl2RenderingContext) {
        gl.delete_texture(Some(&self.gl_texture));
    }
}

pub fn debug_png(png_data: &[u8]) -> Result<(), Error> {
    // Try to load the PNG
    let img = match image::load_from_memory(png_data) {
        Ok(img) => img,
        Err(e) => {
            console::error_1(&format!("Failed to load PNG: {:?}", e).into());
            return Err(Error::ImageLoadError("Invalid PNG data"));
        }
    };

    // Get image info
    let (width, height) = img.dimensions();
    let color_type = img.color();

    console::log_1(&format!("PNG info: {}x{}, color type: {:?}", width, height, color_type).into());

    // Check first few pixels
    let rgba = img.to_rgba8();
    console::log_1(&"First few pixels:".into());
    for y in 0..std::cmp::min(height, 4) {
        for x in 0..std::cmp::min(width, 4) {
            let pixel = rgba.get_pixel(x, y);
            console::log_1(&format!("Pixel ({},{}): RGBA{:?}", x, y, pixel).into());
        }
    }

    Ok(())
}