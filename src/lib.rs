//! High-performance transparent pixel overlay with Rust backend and Node.js API

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoop;

// Module declarations
mod buffer;
mod color;
mod types;
mod window;

// Re-export main types for NAPI compatibility
pub use buffer::*;
pub use color::*;
pub use types::*;

use window::{poll_event_loop, run_event_loop, FrameController, WindowController, WindowState};

/// Application manager for the overlay system
#[napi]
pub struct OverlayApp {
  event_loop: Option<winit::event_loop::EventLoop<()>>,
  windows: Vec<Arc<Mutex<WindowState>>>,
}

impl Default for OverlayApp {
  fn default() -> Self {
    Self::new()
  }
}

#[napi]
impl OverlayApp {
  #[napi(constructor)]
  pub fn new() -> Self {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    Self {
      event_loop: Some(event_loop),
      windows: Vec::new(),
    }
  }

  /// Create a new window with the given configuration
  #[napi]
  pub fn create_window(&mut self, config: WindowConfig) -> Result<OverlayWindow> {
    let event_loop = self
      .event_loop
      .as_mut()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Event loop not available"))?;

    let (window, pixels) = window::create_overlay_window_from_loop(event_loop, &config)?;
    let window_size = window.inner_size();
    let width = window_size.width;
    let height = window_size.height;

    let state = Arc::new(Mutex::new(WindowState {
      pixels: Some(pixels),
      window: Some(window),
      width,
      height,
      event_callback: None,
      render_when_occluded: config.render_when_occluded.unwrap_or(true),
      occluded: false,
    }));

    self.windows.push(state.clone());

    let window_controller = WindowController::new(state.clone());
    let frame_controller = FrameController::new(state.clone());

    Ok(OverlayWindow {
      state,
      window_controller,
      frame_controller,
    })
  }

  /// Poll events once and return (non-blocking)
  #[napi]
  pub fn poll_events(&mut self) -> Result<bool> {
    let event_loop = self
      .event_loop
      .as_mut()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Event loop not available"))?;

    Ok(poll_event_loop(event_loop, &self.windows))
  }

  /// Start the event loop (blocks the current thread)
  #[napi]
  pub fn run(&mut self) -> Result<()> {
    let event_loop = self
      .event_loop
      .take()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Event loop already running"))?;

    if self.windows.is_empty() {
      return Err(Error::new(Status::GenericFailure, "No windows created"));
    }

    run_event_loop(event_loop, self.windows.clone());

    #[allow(unreachable_code)]
    Ok(())
  }
}

/// A wrapper for an overlay window
#[napi]
pub struct OverlayWindow {
  #[allow(dead_code)]
  state: Arc<Mutex<WindowState>>,
  window_controller: WindowController,
  frame_controller: FrameController,
}

#[napi]
impl OverlayWindow {
  /// Register an event callback
  #[napi]
  pub fn on_event(&self, callback: ThreadsafeFunction<OverlayEvent>) -> Result<()> {
    self.window_controller.set_event_callback(callback);
    Ok(())
  }

  /// Update frame with optimized buffer handling
  #[napi]
  pub fn update_frame(&self, buffer: Buffer) -> Result<()> {
    self.frame_controller.update_frame(buffer.as_ref())
  }

  /// Get frame size
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
  pub fn minimize(&self) -> Result<()> {
    self.window_controller.minimize()
  }

  #[napi]
  pub fn maximize(&self) -> Result<()> {
    self.window_controller.maximize()
  }

  #[napi]
  pub fn restore(&self) -> Result<()> {
    self.window_controller.restore()
  }

  #[napi]
  pub fn is_visible(&self) -> Result<bool> {
    self.window_controller.is_visible()
  }

  /// Window positioning
  #[napi]
  pub fn set_position(&self, x: i32, y: i32) -> Result<()> {
    self.window_controller.set_position(x, y)
  }

  #[napi]
  pub fn get_position(&self) -> Result<WindowPosition> {
    self.window_controller.get_position()
  }

  /// Window sizing
  #[napi]
  pub fn set_size(&self, width: u32, height: u32) -> Result<()> {
    self.window_controller.set_size(width, height)
  }

  #[napi]
  pub fn get_size(&self) -> Result<WindowSize> {
    self.window_controller.get_size()
  }

  #[napi]
  pub fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    self.window_controller.set_fullscreen(fullscreen)
  }

  #[napi]
  pub fn is_fullscreen(&self) -> Result<bool> {
    self.window_controller.is_fullscreen()
  }

  /// Window properties
  #[napi]
  pub fn set_title(&self, title: String) -> Result<()> {
    self.window_controller.set_title(&title)
  }

  #[napi]
  pub fn set_window_level(&self, level: WindowLevel) -> Result<()> {
    self.window_controller.set_window_level(level)
  }

  #[napi]
  pub fn request_redraw(&self) -> Result<()> {
    self.window_controller.request_redraw()
  }

  #[napi]
  pub fn set_cursor_visible(&self, visible: bool) -> Result<()> {
    self.window_controller.set_cursor_visible(visible)
  }

  #[napi]
  pub fn set_ignore_mouse_events(&self, ignore: bool) -> Result<()> {
    self.window_controller.set_ignore_mouse_events(ignore)
  }

  #[napi]
  pub fn set_render_when_occluded(&self, render: bool) -> Result<()> {
    self.window_controller.set_render_when_occluded(render);
    Ok(())
  }

  #[napi]
  pub fn is_occluded(&self) -> Result<bool> {
    Ok(self.window_controller.is_occluded())
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
    self
      .frame_controller
      .draw_rectangle(x, y, width, height, &color)
  }

  #[napi]
  pub fn draw_image(&self, x: u32, y: u32, image: DecodedImage) -> Result<()> {
    self.frame_controller.draw_image(x, y, &image)
  }

  /// Get the current frame buffer
  #[napi]
  pub fn get_frame_buffer(&self) -> Result<Buffer> {
    self.frame_controller.get_frame_buffer()
  }

  /// Manually trigger a render
  #[napi]
  pub fn render(&self) -> Result<()> {
    self.frame_controller.render()
  }

  /// Resize the frame buffer and window
  #[napi]
  pub fn resize(&self, width: u32, height: u32) -> Result<()> {
    self.frame_controller.resize(width, height)
  }
}
