/// Minimal safe wrapper around libheif's C API.
///
/// Only exposes what we need: decode a HEIF buffer to interleaved RGB pixels.

use std::os::raw::{c_int, c_void};
use std::ptr;

// =============================================================================
// FFI declarations (libheif C API)
// =============================================================================

#[allow(non_camel_case_types)]
mod ffi {
    use std::os::raw::{c_char, c_int, c_uchar, c_void};

    pub enum heif_context {}
    pub enum heif_image_handle {}
    pub enum heif_image {}

    #[repr(C)]
    pub struct heif_error {
        pub code: c_int,
        pub subcode: c_int,
        pub message: *const c_char,
    }

    impl heif_error {
        pub fn ok(&self) -> bool {
            self.code == 0
        }

        pub fn to_string(&self) -> String {
            if self.message.is_null() {
                return format!("heif error code={} subcode={}", self.code, self.subcode);
            }
            let msg = unsafe { std::ffi::CStr::from_ptr(self.message) };
            msg.to_string_lossy().into_owned()
        }
    }

    // Colorspace and chroma constants
    pub const HEIF_COLORSPACE_RGB: c_int = 1;
    pub const HEIF_CHROMA_INTERLEAVED_RGB: c_int = 10;
    pub const HEIF_CHANNEL_INTERLEAVED: c_int = 10;

    extern "C" {
        pub fn heif_context_alloc() -> *mut heif_context;
        pub fn heif_context_free(ctx: *mut heif_context);

        pub fn heif_context_read_from_memory_without_copy(
            ctx: *mut heif_context,
            data: *const c_void,
            size: usize,
            options: *const c_void,
        ) -> heif_error;

        pub fn heif_context_get_primary_image_handle(
            ctx: *mut heif_context,
            handle: *mut *mut heif_image_handle,
        ) -> heif_error;

        pub fn heif_image_handle_release(handle: *mut heif_image_handle);
        pub fn heif_image_handle_get_width(handle: *const heif_image_handle) -> c_int;
        pub fn heif_image_handle_get_height(handle: *const heif_image_handle) -> c_int;

        pub fn heif_decode_image(
            handle: *const heif_image_handle,
            out_img: *mut *mut heif_image,
            colorspace: c_int,
            chroma: c_int,
            options: *const c_void,
        ) -> heif_error;

        pub fn heif_image_release(img: *mut heif_image);

        pub fn heif_image_get_plane_readonly(
            img: *const heif_image,
            channel: c_int,
            out_stride: *mut c_int,
        ) -> *const c_uchar;
    }
}

// =============================================================================
// Safe wrapper
// =============================================================================

/// Decode a HEIF/HEIC buffer to interleaved RGB pixels.
///
/// Returns (pixels, width, height) where pixels is a Vec<u8> of RGB triplets.
pub fn decode_to_rgb(data: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    unsafe {
        // Allocate context
        let ctx = ffi::heif_context_alloc();
        if ctx.is_null() {
            return Err("Failed to allocate heif context".into());
        }

        // Read from memory (zero-copy — data must outlive the context)
        let err = ffi::heif_context_read_from_memory_without_copy(
            ctx,
            data.as_ptr() as *const c_void,
            data.len(),
            ptr::null(),
        );
        if !err.ok() {
            ffi::heif_context_free(ctx);
            return Err(format!("Failed to read HEIF: {}", err.to_string()));
        }

        // Get primary image handle
        let mut handle: *mut ffi::heif_image_handle = ptr::null_mut();
        let err = ffi::heif_context_get_primary_image_handle(ctx, &mut handle);
        if !err.ok() {
            ffi::heif_context_free(ctx);
            return Err(format!("Failed to get image handle: {}", err.to_string()));
        }

        let width = ffi::heif_image_handle_get_width(handle) as u32;
        let height = ffi::heif_image_handle_get_height(handle) as u32;

        // Decode to interleaved RGB
        let mut image: *mut ffi::heif_image = ptr::null_mut();
        let err = ffi::heif_decode_image(
            handle,
            &mut image,
            ffi::HEIF_COLORSPACE_RGB,
            ffi::HEIF_CHROMA_INTERLEAVED_RGB,
            ptr::null(),
        );
        if !err.ok() {
            ffi::heif_image_handle_release(handle);
            ffi::heif_context_free(ctx);
            return Err(format!("Failed to decode image: {}", err.to_string()));
        }

        // Get raw pixel data
        let mut stride: c_int = 0;
        let plane = ffi::heif_image_get_plane_readonly(
            image,
            ffi::HEIF_CHANNEL_INTERLEAVED,
            &mut stride,
        );
        if plane.is_null() {
            ffi::heif_image_release(image);
            ffi::heif_image_handle_release(handle);
            ffi::heif_context_free(ctx);
            return Err("Failed to get pixel data".into());
        }

        // Copy pixels into a contiguous Vec (stride may include padding)
        let row_bytes = (width * 3) as usize;
        let mut pixels = Vec::with_capacity(row_bytes * height as usize);
        for y in 0..height {
            let row = plane.offset((y as c_int * stride) as isize);
            pixels.extend_from_slice(std::slice::from_raw_parts(row, row_bytes));
        }

        // Cleanup
        ffi::heif_image_release(image);
        ffi::heif_image_handle_release(handle);
        ffi::heif_context_free(ctx);

        Ok((pixels, width, height))
    }
}
