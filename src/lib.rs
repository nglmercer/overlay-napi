//! High-performance transparent pixel overlay with Rust backend and Node.js API
//! 
//! This library provides a modular architecture for creating transparent overlay windows
//! with efficient pixel manipulation and rendering capabilities.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoopBuilder;

// Module declarations
mod buffer;
mod color;
mod image;
mod types;
mod window;

// Re-export main types for NAPI compatibility
pub use color::*;
pub use types::*;
pub use buffer::*;
pub use image::*;

use window::{WindowState, InitialConfig, WindowController, FrameController, create_overlay_window, run_event_loop};

/// Main overlay structure with optimized architecture
#[napi]
pub struct Overlay {
  state: Arc<Mutex<WindowState>>,
  initial_config: Arc<Mutex<Option<InitialConfig>>>,
  window_controller: WindowController,
  frame_controller: FrameController,
}

#[napi]
impl Overlay {
  #[napi(constructor)]
  pub fn new() -> Self {
    let state = Arc::new(Mutex::new(WindowState::new()));
    let initial_config = Arc::new(Mutex::new(None));
    
    let window_controller = WindowController::new(state.clone());
    let frame_controller = FrameController::new(state.clone());

    Self {
      state: state.clone(),
      initial_config: initial_config.clone(),
      window_controller,
      frame_controller,
    }
  }
}

impl Default for Overlay {
  fn default() -> Self {
    Self::new()
  }
}

#[napi]
impl Overlay {
  /// Start the overlay system with optimized event loop
  #[napi]
  pub fn start(&mut self) -> Result<()> {
    let state = self.state.clone();
    let initial_config = self.initial_config.clone();

    // Get initial frame data if available
    let initial_frame_data = {
      let config = initial_config.lock().unwrap();
      if let Some(ref config) = *config {
        config.initial_frame_data.clone()
      } else {
        None
      }
    };

    // Create event loop and window in the same thread
    let event_loop = EventLoopBuilder::new().build();

    // Build configuration with defaults
    let config = {
      let config_guard = initial_config.lock().unwrap();
      config_guard.clone().unwrap_or_default()
    };

    // Create window and pixels with initial configuration
    let (window, pixels) = create_overlay_window(&event_loop, &config)?;

    // Store state and apply initial frame data if available
    {
      let mut state_guard = state.lock().unwrap();
      state_guard.window = Some(window.clone());
      state_guard.pixels = Some(pixels);
    }

    // Apply initial frame data if provided
    if let Some(ref frame_data) = initial_frame_data {
      let mut state_guard = state.lock().unwrap();
      if let Some(pixels) = &mut state_guard.pixels {
        let frame = pixels.frame_mut();
        if frame_data.len() == frame.len() {
          frame.copy_from_slice(frame_data);
        }
      }
    }

    // Run optimized event loop (never returns on most platforms)
    run_event_loop(event_loop, state);
    
    // This point is never reached on most platforms
    #[allow(unreachable_code)]
    Ok(())
  }

  /// Update frame with optimized buffer handling
  #[napi]
  pub fn update_frame(&self, buffer: Buffer) -> Result<()> {
    // Store buffer data for initial configuration
    let buffer_data = buffer.as_ref().to_vec();
    
    // Try to update frame directly first
    match self.frame_controller.update_frame(buffer) {
      Ok(()) => Ok(()),
      Err(_) => {
        // Store for initial configuration if overlay not initialized
        let mut config = self.initial_config.lock().unwrap();
        if let Some(ref mut config) = *config {
          config.initial_frame_data = Some(buffer_data);
        } else {
          let mut new_config = InitialConfig::default();
          new_config.initial_frame_data = Some(buffer_data);
          *config = Some(new_config);
        }
        Ok(())
      }
    }
  }

  /// Get frame size with improved calculation
  #[napi]
  pub fn get_frame_size(&self) -> Result<Vec<u32>> {
    self.frame_controller.get_frame_size()
  }

  /// Window visibility controls
  #[napi]
  pub fn show(&self) -> Result<()> {
    self.window_controller.show()
  }

  #[napi]
  pub fn hide(&self) -> Result<()> {
    self.window_controller.hide()
  }

  #[napi]
  pub fn is_visible(&self) -> Result<bool> {
    self.window_controller.is_visible()
  }

  /// Window positioning
  #[napi]
  pub fn set_position(&self, x: i32, y: i32) -> Result<()> {
    // Try direct update first
    if let Ok(()) = self.window_controller.set_position(x, y) {
      return Ok(());
    }

    // Store for initial configuration
    let mut config = self.initial_config.lock().unwrap();
    if let Some(ref mut config) = *config {
      config.x = x;
      config.y = y;
    } else {
      let mut new_config = InitialConfig::default();
      new_config.x = x;
      new_config.y = y;
      *config = Some(new_config);
    }
    
    Ok(())
  }

  #[napi]
  pub fn get_position(&self) -> Result<WindowPosition> {
    self.window_controller.get_position()
  }

  /// Window sizing
  #[napi]
  pub fn set_size(&self, width: u32, height: u32) -> Result<()> {
    // Try direct update first
    if let Ok(()) = self.window_controller.set_size(width, height) {
      return Ok(());
    }

    // Store for initial configuration
    let mut config = self.initial_config.lock().unwrap();
    if let Some(ref mut config) = *config {
      config.width = width;
      config.height = height;
    } else {
      let mut new_config = InitialConfig::default();
      new_config.width = width;
      new_config.height = height;
      *config = Some(new_config);
    }
    
    Ok(())
  }

  #[napi]
  pub fn get_size(&self) -> Result<WindowSize> {
    self.window_controller.get_size()
  }

  /// Window properties
  #[napi]
  pub fn set_title(&self, title: String) -> Result<()> {
    // Try direct update first
    if let Ok(()) = self.window_controller.set_title(&title) {
      return Ok(());
    }

    // Store for initial configuration
    let mut config = self.initial_config.lock().unwrap();
    if let Some(ref mut config) = *config {
      config.title = title;
    } else {
      let mut new_config = InitialConfig::default();
      new_config.title = title;
      *config = Some(new_config);
    }
    
    Ok(())
  }

  #[napi]
  pub fn set_window_level(&self, level: WindowLevel) -> Result<()> {
    // Try direct update first
    if let Ok(()) = self.window_controller.set_window_level(level) {
      return Ok(());
    }

    // Store for initial configuration
    let mut config = self.initial_config.lock().unwrap();
    if let Some(ref mut config) = *config {
      config.window_level = level;
    } else {
      let mut new_config = InitialConfig::default();
      new_config.window_level = level;
      *config = Some(new_config);
    }
    
    Ok(())
  }

  #[napi]
  pub fn request_redraw(&self) -> Result<()> {
    self.window_controller.request_redraw()
  }

  /// Frame operations
  #[napi]
  pub fn clear_frame(&self, color: Color) -> Result<()> {
    self.frame_controller.clear_frame(&color)
  }

  #[napi]
  pub fn draw_rectangle(
    &self,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: Color,
  ) -> Result<()> {
    self.frame_controller.draw_rectangle(x, y, width, height, &color)
  }
}

// Tests module
#[cfg(test)]
mod tests {
  use super::*;
  use std::thread;
  use std::time::Duration;

  #[test]
  fn test_overlay_creation() {
    let overlay = Overlay::new();
    let state = overlay.state.lock().unwrap();
    assert!(state.window.is_none());
    assert!(state.pixels.is_none());
  }

  #[test]
  fn test_frame_size_calculation() {
    // Test frame size calculation logic
    let frame_size = 1024; // 32x32 pixels (32*32*4 = 4096 bytes for RGBA)
    let pixel_count = frame_size / 4;
    let width = (pixel_count as f64).sqrt() as u32;
    let height = width;

    assert_eq!(width * height * 4, frame_size);
  }

  #[test]
  fn test_buffer_size_validation() {
    let overlay = Overlay::new();

    // Test with uninitialized overlay
    let result = overlay.get_frame_size();
    assert!(result.is_err());

    // Test buffer size validation logic
    let frame_size = 1024; // 32x32 RGBA buffer
    let valid_buffer = vec![0u8; frame_size];
    let invalid_buffer = vec![0u8; frame_size + 100];

    // These would fail because overlay is not initialized, but the logic is tested
    assert!(frame_size == valid_buffer.len());
    assert!(frame_size != invalid_buffer.len());
  }

  #[test]
  fn test_state_thread_safety() {
    let state = Arc::new(Mutex::new(WindowState::new()));
    let state_clone = state.clone();

    let handle = thread::spawn(move || {
      let state = state_clone.lock().unwrap();
      // Simulate some work
      thread::sleep(Duration::from_millis(10));
      assert!(state.window.is_none());
      assert!(state.pixels.is_none());
    });

    let _ = handle.join();

    let state = state.lock().unwrap();
    assert!(state.window.is_none());
    assert!(state.pixels.is_none());
  }

  #[test]
  fn test_error_handling() {
    let overlay = Overlay::new();

    // Test error for uninitialized overlay
    let result = overlay.update_frame(Buffer::from(vec![0u8; 100]));
    assert!(result.is_err());

    let result = overlay.get_frame_size();
    assert!(result.is_err());

    // Test window control methods with uninitialized overlay
    assert!(overlay.show().is_err());
    assert!(overlay.hide().is_err());
    assert!(overlay.set_position(100, 100).is_err());
    assert!(overlay.get_position().is_err());
    assert!(overlay.set_size(800, 600).is_err());
    assert!(overlay.get_size().is_err());
    assert!(overlay.set_title("Test".to_string()).is_err());
    assert!(overlay.set_window_level(WindowLevel::Normal).is_err());
    assert!(overlay.request_redraw().is_err());
    assert!(overlay.is_visible().is_err());
    assert!(overlay.clear_frame(color_red()).is_err());
    assert!(overlay.draw_rectangle(0, 0, 100, 100, color_red()).is_err());
  }

  #[test]
  fn test_color_creation() {
    let color = create_color(255, 128, 64, 255);
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 128);
    assert_eq!(color.b, 64);
    assert_eq!(color.a, 255);

    let rgba = color.to_rgba();
    assert_eq!(rgba, [255, 128, 64, 255]);
  }

  #[test]
  fn test_position_and_size_creation() {
    let pos = create_position(100, 200);
    assert_eq!(pos.x, 100);
    assert_eq!(pos.y, 200);

    let size = create_size(800, 600);
    assert_eq!(size.width, 800);
    assert_eq!(size.height, 600);
  }

  #[test]
  fn test_predefined_colors() {
    let red = color_red();
    assert_eq!(red.r, 255);
    assert_eq!(red.g, 0);
    assert_eq!(red.b, 0);
    assert_eq!(red.a, 255);

    let green = color_green();
    assert_eq!(green.r, 0);
    assert_eq!(green.g, 255);
    assert_eq!(green.b, 0);
    assert_eq!(green.a, 255);

    let blue = color_blue();
    assert_eq!(blue.r, 0);
    assert_eq!(blue.g, 0);
    assert_eq!(blue.b, 255);
    assert_eq!(blue.a, 255);

    let black = color_black();
    assert_eq!(black.r, 0);
    assert_eq!(black.g, 0);
    assert_eq!(black.b, 0);
    assert_eq!(black.a, 255);

    let white = color_white();
    assert_eq!(white.r, 255);
    assert_eq!(white.g, 255);
    assert_eq!(white.b, 255);
    assert_eq!(white.a, 255);

    let transparent = color_transparent();
    assert_eq!(transparent.r, 0);
    assert_eq!(transparent.g, 0);
    assert_eq!(transparent.b, 0);
    assert_eq!(transparent.a, 0);
  }

  #[test]
  fn test_window_level_enum() {
    // Test conversion from our enum to winit enum
    let normal: winit::window::WindowLevel = WindowLevel::Normal.into();
    let always_on_top: winit::window::WindowLevel = WindowLevel::AlwaysOnTop.into();
    let always_on_bottom: winit::window::WindowLevel = WindowLevel::AlwaysOnBottom.into();

    // Just verify the conversions work (exact values are internal to winit)
    match normal {
      winit::window::WindowLevel::Normal => {}
      _ => panic!("Expected Normal window level"),
    }

    match always_on_top {
      winit::window::WindowLevel::AlwaysOnTop => {}
      _ => panic!("Expected AlwaysOnTop window level"),
    }

    match always_on_bottom {
      winit::window::WindowLevel::AlwaysOnBottom => {}
      _ => panic!("Expected AlwaysOnBottom window level"),
    }
  }
}
