use crate::color::Color;
use crate::types::{OverlayEvent, WindowConfig, WindowLevel, WindowPosition, WindowSize};
use napi::bindgen_prelude::Buffer;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::{Error, Result, Status};
use pixels::{Pixels, SurfaceTexture};
use std::sync::{Arc, Mutex};
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Fullscreen, Window, WindowBuilder};

#[cfg(target_os = "windows")]
use winit::platform::run_return::EventLoopExtRunReturn;
#[cfg(target_os = "linux")]
use winit::platform::run_return::EventLoopExtRunReturn;

/// Internal window state management
pub struct WindowState {
  pub pixels: Option<Pixels>,
  pub window: Option<Arc<Window>>,
  pub width: u32,
  pub height: u32,
  pub event_callback: Option<ThreadsafeFunction<OverlayEvent>>,
}

impl WindowState {
  #[allow(dead_code)]
  pub fn new() -> Self {
    Self {
      pixels: None,
      window: None,
      width: 0,
      height: 0,
      event_callback: None,
    }
  }
}

/// Create overlay window with optimized configuration
pub fn create_overlay_window(
  event_loop: &EventLoop<()>,
  config: &WindowConfig,
) -> Result<(Arc<Window>, Pixels)> {
  let width = config.width.unwrap_or(800);
  let height = config.height.unwrap_or(600);
  let title = config
    .title
    .clone()
    .unwrap_or_else(|| "Overlay NAPI".to_string());
  let transparent = config.transparent.unwrap_or(true);
  let decorations = config.decorations.unwrap_or(false);
  let always_on_top = config.always_on_top.unwrap_or(true);
  let resizable = config.resizable.unwrap_or(true);

  // Create transparent overlay window with configuration
  let mut window_builder = WindowBuilder::new()
    .with_transparent(transparent)
    .with_decorations(decorations)
    .with_window_level(if always_on_top {
      winit::window::WindowLevel::AlwaysOnTop
    } else {
      winit::window::WindowLevel::Normal
    })
    .with_title(&title)
    .with_resizable(resizable)
    .with_inner_size(LogicalSize::new(width, height));

  // Set position if specified
  if let (Some(x), Some(y)) = (config.x, config.y) {
    window_builder = window_builder.with_position(LogicalPosition::new(x, y));
  }

  // Set initial state
  if config.fullscreen.unwrap_or(false) {
    window_builder = window_builder.with_fullscreen(Some(Fullscreen::Borderless(None)));
  }
  if config.maximized.unwrap_or(false) {
    window_builder = window_builder.with_maximized(true);
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

  // Set minimized if requested
  if config.minimized.unwrap_or(false) {
    window.set_minimized(true);
  }

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

  pub fn set_event_callback(&self, callback: ThreadsafeFunction<OverlayEvent>) {
    let mut state = self.state.lock().unwrap();
    state.event_callback = Some(callback);
  }

  pub fn show(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_visible(true);
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn hide(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_visible(false);
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn minimize(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_minimized(true);
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn maximize(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_maximized(true);
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn restore(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_minimized(false);
      window.set_maximized(false);
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn set_fullscreen(&self, fullscreen: bool) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      if fullscreen {
        window.set_fullscreen(Some(Fullscreen::Borderless(None)));
      } else {
        window.set_fullscreen(None);
      }
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn is_fullscreen(&self) -> Result<bool> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      Ok(window.fullscreen().is_some())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn set_position(&self, x: i32, y: i32) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_outer_position(LogicalPosition::new(x, y));
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
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
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn set_size(&self, width: u32, height: u32) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_inner_size(LogicalSize::new(width, height));
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
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
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn set_title(&self, title: &str) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_title(title);
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn set_window_level(&self, level: WindowLevel) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_window_level(level.into());
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn is_visible(&self) -> Result<bool> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      Ok(window.is_visible().unwrap_or(false))
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn request_redraw(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.request_redraw();
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn set_cursor_visible(&self, visible: bool) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_cursor_visible(visible);
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn set_ignore_mouse_events(&self, ignore: bool) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      window.set_cursor_hittest(!ignore).map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Failed to set hittest: {}", e),
        )
      })?;
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
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

  pub fn update_frame(&self, buffer_data: &[u8]) -> Result<()> {
    let mut state = self.state.lock().unwrap();

    if let Some(pixels) = &mut state.pixels {
      let frame = pixels.frame_mut();

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

      frame.copy_from_slice(buffer_data);

      if let Some(window) = &state.window {
        window.request_redraw();
      }
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn get_frame_size(&self) -> Result<Vec<u32>> {
    let state = self.state.lock().unwrap();
    if state.pixels.is_some() {
      Ok(vec![state.width, state.height])
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn clear_frame(&self, color: &Color) -> Result<()> {
    let mut state = self.state.lock().unwrap();
    if let Some(pixels) = &mut state.pixels {
      let frame = pixels.frame_mut();
      let rgba = color.to_rgba();
      for chunk in frame.chunks_exact_mut(4) {
        chunk.copy_from_slice(&rgba);
      }
      if let Some(window) = &state.window {
        window.request_redraw();
      }
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
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
    let frame_width = state.width as usize;
    let frame_height = state.height as usize;

    if let Some(pixels) = &mut state.pixels {
      let frame = pixels.frame_mut();

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

      if let Some(window) = &state.window {
        window.request_redraw();
      }
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn draw_image(&self, x: u32, y: u32, image: &crate::types::DecodedImage) -> Result<()> {
    let mut state = self.state.lock().unwrap();
    let frame_width = state.width as usize;
    let frame_height = state.height as usize;

    if let Some(pixels) = &mut state.pixels {
      let frame = pixels.frame_mut();
      let img_data = image.data.as_ref();
      let img_width = image.width as usize;
      let img_height = image.height as usize;

      // Simple blit with bounds checking
      for iy in 0..img_height {
        let py = y as usize + iy;
        if py >= frame_height {
          break;
        }

        for ix in 0..img_width {
          let px = x as usize + ix;
          if px >= frame_width {
            break;
          }

          let src_idx = (iy * img_width + ix) * 4;
          let dst_idx = (py * frame_width + px) * 4;

          if src_idx + 3 < img_data.len() && dst_idx + 3 < frame.len() {
            frame[dst_idx..dst_idx + 4].copy_from_slice(&img_data[src_idx..src_idx + 4]);
          }
        }
      }

      if let Some(window) = &state.window {
        window.request_redraw();
      }
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn get_frame_buffer(&self) -> Result<Buffer> {
    let state = self.state.lock().unwrap();
    if let Some(pixels) = &state.pixels {
      let frame = pixels.frame();
      Ok(Buffer::from(frame.to_vec()))
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn render(&self) -> Result<()> {
    let state = self.state.lock().unwrap();
    if let Some(pixels) = &state.pixels {
      pixels
        .render()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to render: {}", e)))?;
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }

  pub fn resize(&self, width: u32, height: u32) -> Result<()> {
    let mut state = self.state.lock().unwrap();
    if let Some(pixels) = &mut state.pixels {
      pixels
        .resize_buffer(width, height)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to resize: {}", e)))?;

      state.width = width;
      state.height = height;

      if let Some(window) = &state.window {
        window.request_redraw();
      }
      Ok(())
    } else {
      Err(Error::new(Status::GenericFailure, "Window not initialized"))
    }
  }
}

/// Event loop runner
pub fn run_event_loop(event_loop: EventLoop<()>, windows: Vec<Arc<Mutex<WindowState>>>) -> ! {
  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    handle_winit_event(event, control_flow, &windows);
  });
}

pub fn poll_event_loop(
  event_loop: &mut EventLoop<()>,
  windows: &[Arc<Mutex<WindowState>>],
) -> bool {
  let mut app_should_exit = false;
  #[cfg(any(target_os = "windows", target_os = "linux"))]
  {
    event_loop.run_return(|event, _, control_flow| match event {
      Event::RedrawEventsCleared => {
        *control_flow = ControlFlow::Exit;
      }
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => {
        app_should_exit = true;
        *control_flow = ControlFlow::Exit;
      }
      _ => {
        *control_flow = ControlFlow::Poll;
        handle_winit_event(event, control_flow, windows);
      }
    });
  }
  app_should_exit
}

fn handle_winit_event(
  event: Event<()>,
  control_flow: &mut ControlFlow,
  windows: &[Arc<Mutex<WindowState>>],
) {
  match event {
    Event::WindowEvent {
      event, window_id, ..
    } => {
      let target_window = windows.iter().find(|w| {
        let guard = w.lock().unwrap();
        guard
          .window
          .as_ref()
          .map(|win| win.id() == window_id)
          .unwrap_or(false)
      });

      if let Some(state_arc) = target_window {
        let mut overlay_event = None;

        match event {
          WindowEvent::CloseRequested => {
            overlay_event = Some(OverlayEvent::CloseRequested);
            *control_flow = ControlFlow::Exit;
          }
          WindowEvent::Resized(size) => {
            overlay_event = Some(OverlayEvent::Resized);
            let mut state = state_arc.lock().unwrap();
            if let Some(pixels) = &mut state.pixels {
              let _ = pixels.resize_buffer(size.width, size.height);
            }
          }
          WindowEvent::Moved(_) => {
            overlay_event = Some(OverlayEvent::Moved);
          }
          WindowEvent::Focused(focused) => {
            overlay_event = Some(if focused {
              OverlayEvent::Focused
            } else {
              OverlayEvent::Blurred
            });
          }
          WindowEvent::CursorEntered { .. } => {
            overlay_event = Some(OverlayEvent::MouseEnter);
          }
          WindowEvent::CursorLeft { .. } => {
            overlay_event = Some(OverlayEvent::MouseLeave);
          }
          _ => {}
        }

        if let Some(ev) = overlay_event {
          let state = state_arc.lock().unwrap();
          if let Some(cb) = &state.event_callback {
            cb.call(Ok(ev), ThreadsafeFunctionCallMode::NonBlocking);
          }
        }
      }
    }
    Event::RedrawRequested(window_id) => {
      let target_window = windows.iter().find(|w| {
        let guard = w.lock().unwrap();
        guard
          .window
          .as_ref()
          .map(|win| win.id() == window_id)
          .unwrap_or(false)
      });

      if let Some(state_arc) = target_window {
        let mut state = state_arc.lock().unwrap();
        if let Some(pixels) = &mut state.pixels {
          if pixels.render().is_err() {
            // eprintln!("Failed to render");
          }
        }
      }
    }
    _ => {}
  }
}
