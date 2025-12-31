//! Image loading and processing utilities

use crate::types::DecodedImage;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Load and decode image from file path
#[napi]
pub fn load_image(path: String) -> Result<DecodedImage> {
  let img = image::open(&path).map_err(|e| {
    Error::new(
      Status::GenericFailure,
      format!("Failed to open image: {}", e),
    )
  })?;

  // Convert to RGBA8 format for consistent processing
  let img = img.to_rgba8();
  let (width, height) = img.dimensions();
  let data = img.into_raw();

  Ok(DecodedImage {
    data: Buffer::from(data),
    width,
    height,
  })
}

/// Load image from memory buffer
pub fn load_image_from_memory(data: &[u8]) -> std::result::Result<DecodedImage, String> {
  let img = image::load_from_memory(data).map_err(|e| format!("Failed to load image: {}", e))?;

  let img = img.to_rgba8();
  let (width, height) = img.dimensions();
  let pixel_data = img.into_raw();

  Ok(DecodedImage {
    data: Buffer::from(pixel_data),
    width,
    height,
  })
}

/// Resize image to target dimensions
pub fn resize_image(
  image_data: &[u8],
  original_width: u32,
  original_height: u32,
  target_width: u32,
  target_height: u32,
) -> std::result::Result<Vec<u8>, String> {
  use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};

  // Create image buffer from raw data
  let img =
    ImageBuffer::<Rgba<u8>, _>::from_raw(original_width, original_height, image_data.to_vec())
      .ok_or_else(|| "Invalid image dimensions or data".to_string())?;

  // Convert to DynamicImage for resizing
  let dynamic_img = DynamicImage::ImageRgba8(img);

  // Resize using high-quality Lanczos3 filtering
  let resized = dynamic_img.resize(target_width, target_height, image::imageops::Lanczos3);

  // Extract raw pixel data
  let (_width, _height) = resized.dimensions();
  let resized_data = resized.to_rgba8().into_raw();

  Ok(resized_data)
}

/// Apply alpha blending to image
pub fn blend_with_background(image_data: &mut [u8], background_color: &crate::color::Color) {
  let bg_rgba = background_color.to_rgba();

  for pixel in image_data.chunks_exact_mut(4) {
    let alpha = pixel[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    pixel[0] = (pixel[0] as f32 * alpha + bg_rgba[0] as f32 * inv_alpha) as u8;
    pixel[1] = (pixel[1] as f32 * alpha + bg_rgba[1] as f32 * inv_alpha) as u8;
    pixel[2] = (pixel[2] as f32 * alpha + bg_rgba[2] as f32 * inv_alpha) as u8;
    pixel[3] = 255; // Fully opaque after blending
  }
}

/// Convert between different pixel formats
pub fn convert_pixel_format(
  data: &[u8],
  from_format: PixelFormat,
  to_format: PixelFormat,
  width: u32,
  height: u32,
) -> std::result::Result<Vec<u8>, String> {
  match (from_format, to_format) {
    (PixelFormat::RGB, PixelFormat::RGBA) => {
      // RGB to RGBA conversion
      let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

      for rgb in data.chunks_exact(3) {
        rgba_data.extend_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
      }

      Ok(rgba_data)
    }
    (PixelFormat::RGBA, PixelFormat::RGB) => {
      // RGBA to RGB conversion (drop alpha channel)
      let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);

      for rgba in data.chunks_exact(4) {
        rgb_data.extend_from_slice(&[rgba[0], rgba[1], rgba[2]]);
      }

      Ok(rgb_data)
    }
    (PixelFormat::BGRA, PixelFormat::RGBA) => {
      // BGRA to RGBA conversion (swap R and B channels)
      let mut rgba_data = Vec::with_capacity(data.len());

      for bgra in data.chunks_exact(4) {
        rgba_data.extend_from_slice(&[bgra[2], bgra[1], bgra[0], bgra[3]]);
      }

      Ok(rgba_data)
    }
    _ => Err(format!(
      "Unsupported format conversion: {:?} to {:?}",
      from_format, to_format
    )),
  }
}

/// Supported pixel formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelFormat {
  RGB,
  RGBA,
  BGRA,
  BGR,
}

/// Image processing configuration
#[derive(Debug, Clone)]
pub struct ImageProcessingConfig {
  pub resize_filter: ResizeFilter,
  pub maintain_aspect_ratio: bool,
  pub background_color: Option<crate::color::Color>,
}

impl Default for ImageProcessingConfig {
  fn default() -> Self {
    Self {
      resize_filter: ResizeFilter::Lanczos3,
      maintain_aspect_ratio: true,
      background_color: None,
    }
  }
}

/// Available resize filters
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResizeFilter {
  Nearest,
  Triangle,
  CatmullRom,
  Gaussian,
  Lanczos3,
}

impl ResizeFilter {
  pub fn to_image_filter(&self) -> image::imageops::FilterType {
    match self {
      ResizeFilter::Nearest => image::imageops::Nearest,
      ResizeFilter::Triangle => image::imageops::Triangle,
      ResizeFilter::CatmullRom => image::imageops::CatmullRom,
      ResizeFilter::Gaussian => image::imageops::Gaussian,
      ResizeFilter::Lanczos3 => image::imageops::Lanczos3,
    }
  }
}
