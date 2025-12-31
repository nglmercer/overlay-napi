//! Common types and structures for the overlay system

use napi::bindgen_prelude::*;
use napi_derive::napi;
use winit::window::WindowLevel as WinitWindowLevel;

#[napi]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowLevel {
  Normal,
  AlwaysOnTop,
  AlwaysOnBottom,
}

impl From<WindowLevel> for WinitWindowLevel {
  fn from(level: WindowLevel) -> Self {
    match level {
      WindowLevel::Normal => WinitWindowLevel::Normal,
      WindowLevel::AlwaysOnTop => WinitWindowLevel::AlwaysOnTop,
      WindowLevel::AlwaysOnBottom => WinitWindowLevel::AlwaysOnBottom,
    }
  }
}

#[napi(object)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowPosition {
  pub x: i32,
  pub y: i32,
}

#[napi(object)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowSize {
  pub width: u32,
  pub height: u32,
}

#[napi(object)]
#[derive(Debug, Clone, PartialEq)]
pub struct LineParams {
  pub x1: u32,
  pub y1: u32,
  pub x2: u32,
  pub y2: u32,
  pub buffer_width: u32,
  pub buffer_height: u32,
  pub color: crate::color::Color,
}

#[napi(object)]
pub struct DecodedImage {
  pub data: Buffer,
  pub width: u32,
  pub height: u32,
}

impl Clone for DecodedImage {
  fn clone(&self) -> Self {
    Self {
      data: Buffer::from(self.data.as_ref().to_vec()),
      width: self.width,
      height: self.height,
    }
  }
}

// Constructor functions
#[napi]
pub fn create_position(x: i32, y: i32) -> WindowPosition {
  WindowPosition { x, y }
}

#[napi]
pub fn create_size(width: u32, height: u32) -> WindowSize {
  WindowSize { width, height }
}
