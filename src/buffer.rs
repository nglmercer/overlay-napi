//! Buffer manipulation and drawing utilities

use crate::color::Color;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Optimized buffer size calculation
#[inline]
pub fn calculate_buffer_size(width: u32, height: u32) -> usize {
  (width as usize) * (height as usize) * 4 // RGBA = 4 bytes per pixel
}

/// Fill buffer with solid color using SIMD-like optimization
pub fn fill_buffer_rgba_optimized(buffer: &[u8], r: u8, g: u8, b: u8, a: u8) -> Vec<u8> {
  let len = buffer.len();

  // Initialize with zeros, then fill with color
  let mut new_data = vec![0u8; len];

  // Fill with optimized chunk processing
  for chunk in new_data.chunks_exact_mut(4) {
    chunk[0] = r;
    chunk[1] = g;
    chunk[2] = b;
    chunk[3] = a;
  }

  new_data
}

/// Clear buffer with solid color in-place
pub fn clear_buffer_optimized(frame: &mut [u8], _width: u32, _height: u32, color: &Color) {
  let rgba = color.to_rgba();
  for chunk in frame.chunks_exact_mut(4) {
    chunk.copy_from_slice(&rgba);
  }
}

/// Parameters for rectangle drawing
pub struct RectangleParams {
  pub x: u32,
  pub y: u32,
  pub width: u32,
  pub height: u32,
  pub frame_width: usize,
  pub frame_height: usize,
}

/// Draw rectangle with bounds checking and optimization
pub fn draw_rectangle_optimized(frame: &mut [u8], params: RectangleParams, color: &Color) {
  let RectangleParams {
    x,
    y,
    width,
    height,
    frame_width,
    frame_height,
  } = params;
  let rgba = color.to_rgba();

  // Calculate bounds with clamping
  let start_x = x.min(frame_width as u32);
  let start_y = y.min(frame_height as u32);
  let end_x = x.saturating_add(width).min(frame_width as u32);
  let end_y = y.saturating_add(height).min(frame_height as u32);

  // Optimized rectangle drawing
  for py in start_y..end_y {
    let row_start = (py as usize * frame_width + start_x as usize) * 4;
    let row_end = (py as usize * frame_width + end_x as usize) * 4;

    for index in (row_start..row_end).step_by(4) {
      frame[index..index + 4].copy_from_slice(&rgba);
    }
  }
}

/// Draw pixel with bounds checking
#[inline]
pub fn draw_pixel_safe(
  buffer: &mut [u8],
  x: u32,
  y: u32,
  width: u32,
  color: &Color,
) -> std::result::Result<(), String> {
  let index = (y * width + x) as usize * 4;

  if index + 3 < buffer.len() {
    let rgba = color.to_rgba();
    buffer[index..index + 4].copy_from_slice(&rgba);
    std::result::Result::Ok(())
  } else {
    std::result::Result::Err("Pixel position out of bounds".to_string())
  }
}

/// Bresenham's line algorithm with optimization
pub fn draw_line_optimized(buffer: &mut [u8], params: crate::types::LineParams, color: &Color) {
  let crate::types::LineParams {
    x1,
    y1,
    x2,
    y2,
    buffer_width,
    buffer_height,
    ..
  } = params;
  let rgba = color.to_rgba();

  // Bresenham's line algorithm
  let mut x0 = x1 as i32;
  let mut y0 = y1 as i32;
  let x1_i = x2 as i32;
  let y1_i = y2 as i32;

  let dx = (x1_i - x0).abs();
  let dy = -(y1_i - y0).abs();
  let mut error = dx + dy;

  let sx = if x0 < x1_i { 1 } else { -1 };
  let sy = if y0 < y1_i { 1 } else { -1 };

  loop {
    // Draw pixel at current position with bounds checking
    if x0 >= 0 && y0 >= 0 && (x0 as u32) < buffer_width && (y0 as u32) < buffer_height {
      let index = (y0 as u32 * buffer_width + x0 as u32) as usize * 4;
      if index + 3 < buffer.len() {
        buffer[index..index + 4].copy_from_slice(&rgba);
      }
    }

    if x0 == x1_i && y0 == y1_i {
      break;
    }

    let e2 = 2 * error;
    if e2 >= dy {
      error += dy;
      x0 += sx;
    }
    if e2 <= dx {
      error += dx;
      y0 += sy;
    }
  }
}

/// Bresenham's circle algorithm with optimization
pub fn draw_circle_optimized(
  buffer: &mut [u8],
  cx: u32,
  cy: u32,
  radius: u32,
  buffer_width: u32,
  buffer_height: u32,
  color: &Color,
) {
  let rgba = color.to_rgba();
  let radius_i = radius as i32;
  let cx_i = cx as i32;
  let cy_i = cy as i32;

  // Bresenham's circle algorithm
  let mut x = 0i32;
  let mut y = radius_i;
  let mut d = 3 - 2 * radius_i;

  while y >= x {
    // Draw 8 symmetric points
    let points = [
      (cx_i + x, cy_i + y),
      (cx_i - x, cy_i + y),
      (cx_i + x, cy_i - y),
      (cx_i - x, cy_i - y),
      (cx_i + y, cy_i + x),
      (cx_i - y, cy_i + x),
      (cx_i + y, cy_i - x),
      (cx_i - y, cy_i - x),
    ];

    for (px, py) in points {
      if px >= 0 && py >= 0 && (px as u32) < buffer_width && (py as u32) < buffer_height {
        let index = (py as u32 * buffer_width + px as u32) as usize * 4;
        if index + 3 < buffer.len() {
          buffer[index..index + 4].copy_from_slice(&rgba);
        }
      }
    }

    x += 1;
    if d > 0 {
      y -= 1;
      d += 4 * (x - y) + 10;
    } else {
      d += 4 * x + 6;
    }
  }
}

// NAPI exports
#[napi]
pub fn calculate_buffer_size_napi(width: u32, height: u32) -> u32 {
  calculate_buffer_size(width, height) as u32
}

#[napi]
pub fn create_rgba_buffer(width: u32, height: u32) -> Buffer {
  let size = calculate_buffer_size(width, height);
  let data = vec![0u8; size];
  Buffer::from(data)
}

#[napi]
pub fn fill_buffer_color(buffer: Buffer, color: Color) -> Result<Buffer> {
  let new_data = fill_buffer_rgba_optimized(buffer.as_ref(), color.r, color.g, color.b, color.a);
  Ok(Buffer::from(new_data))
}

#[napi]
pub fn draw_pixel(buffer: Buffer, x: u32, y: u32, width: u32, color: Color) -> Result<Buffer> {
  let buffer_data = buffer.as_ref();
  let mut new_data = buffer_data.to_vec();

  match draw_pixel_safe(&mut new_data, x, y, width, &color) {
    Ok(_) => Ok(Buffer::from(new_data)),
    Err(e) => Err(Error::new(Status::InvalidArg, e)),
  }
}

#[napi]
pub fn draw_line(buffer: Buffer, params: crate::types::LineParams) -> Result<Buffer> {
  let buffer_data = buffer.as_ref();
  let mut new_data = buffer_data.to_vec();

  draw_line_optimized(&mut new_data, params.clone(), &params.color);

  Ok(Buffer::from(new_data))
}

#[napi]
pub fn draw_circle(
  buffer: Buffer,
  cx: u32,
  cy: u32,
  radius: u32,
  buffer_width: u32,
  buffer_height: u32,
  color: Color,
) -> Result<Buffer> {
  let buffer_data = buffer.as_ref();
  let mut new_data = buffer_data.to_vec();

  draw_circle_optimized(
    &mut new_data,
    cx,
    cy,
    radius,
    buffer_width,
    buffer_height,
    &color,
  );

  Ok(Buffer::from(new_data))
}
