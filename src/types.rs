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

#[napi]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OverlayEvent {
  Resized,
  Moved,
  CloseRequested,
  Destroyed,
  Focused,
  Blurred,
  Minimized,
  Maximized,
  Restored,
  MouseEnter,
  MouseLeave,
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

#[napi(object)]
#[derive(Debug, Clone)]
pub struct WindowConfig {
  pub width: Option<u32>,
  pub height: Option<u32>,
  pub x: Option<i32>,
  pub y: Option<i32>,
  pub title: Option<String>,
  pub always_on_top: Option<bool>,
  pub transparent: Option<bool>,
  pub decorations: Option<bool>,
  pub resizable: Option<bool>,
  pub fullscreen: Option<bool>,
  pub minimized: Option<bool>,
  pub maximized: Option<bool>,
  pub render_when_occluded: Option<bool>,
  pub handle_event_loop_modal: Option<bool>,
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

#[napi]
pub fn create_window_config() -> WindowConfig {
  WindowConfig {
    width: Some(800),
    height: Some(600),
    x: Some(100),
    y: Some(100),
    title: Some("Overlay NAPI".to_string()),
    always_on_top: Some(true),
    transparent: Some(true),
    decorations: Some(false),
    resizable: Some(true),
    fullscreen: Some(false),
    minimized: Some(false),
    maximized: Some(false),
    render_when_occluded: Some(true),
    handle_event_loop_modal: Some(true),
  }
}
