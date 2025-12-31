//! Window management and event handling

use crate::color::Color;
use crate::types::{WindowLevel, WindowPosition, WindowSize};
use napi::bindgen_prelude::Buffer;
use napi::{Error, Result, Status};
use pixels::{Pixels, SurfaceTexture};
use std::sync::{Arc, Mutex};
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

/// Internal window state management
pub struct WindowState {
  pub pixels: Option<Pixels>,
  pub window: Option<Arc<Window>>,
}

impl WindowState {
  pub fn new() -> Self {
    Self {
      pixels: None,
      window: None,
    }
  }
}

/// Initial configuration for window creation
#[derive(Debug, Clone)]
pub struct InitialConfig {
  pub width: u32,
  pub height: u32,
  pub x: i32,
  pub y: i32,
  pub title: String,
  pub window_level: WindowLevel,
  pub initial_frame_data: Option<Vec<u8>>,
}

impl Default for InitialConfig {
  fn default() -> Self {
    Self {
      width: 800,
      height: 600,
      x: 100,
      y: 100,
      title: "Overlay NAPI".to_string(),
      window_level: WindowLevel::AlwaysOnTop,
      initial_frame_data: None,
    }
  }
}

/// Create overlay window with optimized configuration
pub fn create_overlay_window(
  event_loop: &EventLoop<()>,
  config: &InitialConfig,
) -> Result<(Arc<Window>, Pixels)> {
  // Create transparent overlay window with configuration
  let mut window_builder = WindowBuilder::new()
    .with_transparent(true)
    .with_decorations(false)
    .with_window_level(config.window_level.into())
    .with_title(&config.title)
    .with_inner_size(LogicalSize::new(config.width, config.height));

  // Set position if specified
  if config.x != 0 || config.y != 0 {
    window_builder = window_builder.with_position(LogicalPosition::new(config.x, config.y));
  }

  let window = Arc::new(window_builder.build(event_loop).map_err(|e| {
    Error::new(
      Status::GenericFailure,
      format!("Failed to create window: {}", e),
    )
  })?);

  // Get window size
  let window_size = window.inner_size();

  // Create pixels surface - use raw window handle for compatibility
  let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
  let pixels =
    Pixels::new(window_size.width, window_size.height, surface_texture).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to create pixels: {}", e),
      )
    })?;

  Ok((window, pixels))
}

/// Window control operations
pub struct WindowController {
  state: Arc<Mutex<WindowState>>,
}

impl WindowController {
  pub fn new(state: Arc<Mutex<WindowState>>) -> Self {
    Self { state }
  }

  pub fn show(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_visible(true);
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn hide(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_visible(false);
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn set_position(&self, x: i32, y: i32) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_outer_position(LogicalPosition::new(x, y));
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn get_position(&self) -> Result<WindowPosition> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      let pos = window.outer_position().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to get position: {}", e),
        )
      })?;
      Ok(WindowPosition { x: pos.x, y: pos.y })
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn set_size(&self, width: u32, height: u32) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_inner_size(LogicalSize::new(width, height));
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn get_size(&self) -> Result<WindowSize> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      let size = window.inner_size();
      Ok(WindowSize {
        width: size.width,
        height: size.height,
      })
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn set_title(&self, title: &str) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_title(title);
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn set_window_level(&self, level: WindowLevel) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_window_level(level.into());
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn is_visible(&self) -> Result<bool> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      Ok(window.is_visible().unwrap_or(false))
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn request_redraw(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.request_redraw();
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }
}

/// Frame buffer operations
pub struct FrameController {
  state: Arc<Mutex<WindowState>>,
}

impl FrameController {
  pub fn new(state: Arc<Mutex<WindowState>>) -> Self {
    Self { state }
  }

  pub fn update_frame(&self, buffer: Buffer) -> Result<()> {
    let mut state = self.state.lock().unwrap();

    if let Some(pixels) = &mut state.pixels {
      let frame = pixels.frame_mut();
      let buffer_data = buffer.as_ref();

      // Ensure buffer size matches frame size
      if buffer_data.len() != frame.len() {
        return Err(Error::new(
          Status::InvalidArg,
          format!(
            "Buffer size mismatch: expected {} bytes, got {} bytes",
            frame.len(),
            buffer_data.len()
          ),
        ));
      }

      // Copy buffer data to frame
      frame.copy_from_slice(buffer_data);
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn get_frame_size(&self) -> Result<Vec<u32>> {
    let state = self.state.lock().unwrap();

    if let Some(pixels) = &state.pixels {
      let frame = pixels.frame();
      let size = frame.len() / 4; // RGBA = 4 bytes per pixel
      let width = (size as f64).sqrt() as u32;
      let height = width;

      Ok(vec![width, height])
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn clear_frame(&self, color: &Color) -> Result<()> {
    let mut state = self.state.lock().unwrap();

    if let Some(pixels) = &mut state.pixels {
      let frame = pixels.frame_mut();
      let rgba = color.to_rgba();

      // Fill frame with solid color using optimized chunks
      for chunk in frame.chunks_exact_mut(4) {
        chunk.copy_from_slice(&rgba);
      }
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  pub fn draw_rectangle(
    &self,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: &Color,
  ) -> Result<()> {
    let mut state = self.state.lock().unwrap();

    if let Some(pixels) = &mut state.pixels {
      let frame = pixels.frame_mut();
      let frame_size = self.get_frame_size()?;
      let frame_width = frame_size[0] as usize;
      let frame_height = frame_size[1] as usize;

      crate::buffer::draw_rectangle_optimized(
        frame,
        crate::buffer::RectangleParams {
          x,
          y,
          width,
          height,
          frame_width,
          frame_height,
        },
        color,
      );
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  /// Get the current frame buffer for advanced manipulations
  pub fn get_frame_buffer(&self) -> Result<Buffer> {
    let state = self.state.lock().unwrap();

    if let Some(pixels) = &state.pixels {
      let frame = pixels.frame();
      Ok(Buffer::from(frame.to_vec()))
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  /// Manually trigger a render of the current frame
  pub fn render(&self) -> Result<()> {
    let state = self.state.lock().unwrap();

    if let Some(pixels) = &state.pixels {
      pixels
        .render()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to render: {}", e)))?;
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }

  /// Resize the frame buffer and window
  pub fn resize(&self, width: u32, height: u32) -> Result<()> {
    let mut state = self.state.lock().unwrap();

    if let Some(pixels) = &mut state.pixels {
      pixels
        .resize_buffer(width, height)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to resize: {}", e)))?;
      Ok(())
    } else {
      Err(Error::new(
        Status::GenericFailure,
        "Overlay not initialized",
      ))
    }
  }
}

/// Event loop runner with optimized event handling
pub fn run_event_loop(event_loop: EventLoop<()>, state: Arc<Mutex<WindowState>>) -> ! {
  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent { event, .. } => {
        match event {
          WindowEvent::CloseRequested => {
            *control_flow = ControlFlow::Exit;
          }
          WindowEvent::Resized(size) => {
            let mut state_guard = state.lock().unwrap();
            if let Some(pixels) = &mut state_guard.pixels {
              // Handle resize
              let _ = pixels.resize_buffer(size.width, size.height);
            }
          }
          _ => {}
        }
      }
      Event::RedrawRequested(_) => {
        let mut state_guard = state.lock().unwrap();
        if let Some(pixels) = &mut state_guard.pixels {
          // Render the current frame
          if pixels.render().is_err() {
            eprintln!("Failed to render frame");
          }
        }
      }
      Event::MainEventsCleared => {
        // Request redraw on each frame - avoid double lock
        let state_guard = state.lock().unwrap();
        if let Some(window) = &state_guard.window {
          window.request_redraw();
        }
      }
      _ => {}
    }
  });
}
