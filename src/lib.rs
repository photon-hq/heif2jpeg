use napi::bindgen_prelude::*;
use napi_derive::napi;

mod heif;

// =============================================================================
// Async task — runs conversion on libuv thread pool
// =============================================================================

pub struct ConvertTask {
    input: Vec<u8>,
    quality: u8,
}

impl napi::Task for ConvertTask {
    type Output = Vec<u8>;
    type JsValue = Buffer;

    fn compute(&mut self) -> Result<Self::Output> {
        convert(&self.input, self.quality)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output.into())
    }
}

fn convert(input: &[u8], quality: u8) -> Result<Vec<u8>> {
    // Decode HEIF → raw RGB pixels
    let (pixels, width, height) =
        heif::decode_to_rgb(input).map_err(|e| Error::from_reason(e))?;

    // Encode RGB pixels → JPEG (pure Rust)
    let mut buf = Vec::new();
    let encoder = jpeg_encoder::Encoder::new(&mut buf, quality);
    encoder
        .encode(
            &pixels,
            width as u16,
            height as u16,
            jpeg_encoder::ColorType::Rgb,
        )
        .map_err(|e| Error::from_reason(format!("JPEG encode failed: {e}")))?;
    Ok(buf)
}

// =============================================================================
// N-API exports
// =============================================================================

#[napi(object)]
pub struct ConvertOptions {
    /// JPEG quality (1-100, default 85)
    pub quality: Option<u32>,
}

/// Convert a HEIF/HEIC buffer to JPEG.
#[napi(ts_return_type = "Promise<Buffer>")]
pub fn heif_to_jpeg(input: Uint8Array, options: Option<ConvertOptions>) -> AsyncTask<ConvertTask> {
    let quality = options
        .and_then(|o| o.quality)
        .unwrap_or(85)
        .clamp(1, 100) as u8;

    AsyncTask::new(ConvertTask {
        input: input.to_vec(),
        quality,
    })
}
