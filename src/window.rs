use crate::color::Color;
use crate::types::{OverlayEvent, WindowConfig, WindowLevel, WindowPosition, WindowSize};
use napi::bindgen_prelude::Buffer;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi::{Error, Result, Status};
use pixels::{Pixels, SurfaceTexture};
use std::sync::{Arc, Mutex};
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

#[cfg(target_os = "windows")]
use winit::platform::pump_events::EventLoopExtPumpEvents;

pub struct WindowState {
  pub pixels: Option<Pixels<'static>>,
  pub window: Option<Arc<Window>>,
  pub width: u32,
  pub height: u32,
  pub event_callback: Option<ThreadsafeFunction<OverlayEvent>>,
  pub render_when_occluded: bool,
  pub occluded: bool,
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
      render_when_occluded: true,
      occluded: false,
    }
  }
}

pub struct OverlayApplication<'a> {
  pub windows: &'a [Arc<Mutex<WindowState>>],
  pub exit_requested: bool,
}

impl<'a> ApplicationHandler for OverlayApplication<'a> {
  fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

  fn window_event(
    &mut self,
    event_loop: &ActiveEventLoop,
    window_id: WindowId,
    event: WindowEvent,
  ) {
    let target_window = self.windows.iter().find(|w| {
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
          self.exit_requested = true;
          event_loop.exit();
        }
        WindowEvent::Resized(size) => {
          overlay_event = Some(OverlayEvent::Resized);
          let mut state = state_arc.lock().unwrap();
          state.width = size.width;
          state.height = size.height;
          if let Some(pixels) = &mut state.pixels {
            let _ = pixels.resize_surface(size.width, size.height);
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
        WindowEvent::Occluded(occluded) => {
          overlay_event = Some(if occluded {
            OverlayEvent::Minimized
          } else {
            OverlayEvent::Restored
          });
          let mut state = state_arc.lock().unwrap();
          state.occluded = occluded;
        }
        WindowEvent::RedrawRequested => {
          let mut state = state_arc.lock().unwrap();
          if let Some(pixels) = &mut state.pixels {
            if pixels.render().is_err() {
              // eprintln!("Failed to render");
            }
          }
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
}

/// Create overlay window from Loop
pub fn create_overlay_window_from_loop(
  event_loop: &mut EventLoop<()>,
  config: &WindowConfig,
) -> Result<(Arc<Window>, Pixels<'static>)> {
  let mut window_pixels = None;

  {
    struct Loader<'a> {
      config: &'a WindowConfig,
      result: &'a mut Option<Result<(Arc<Window>, Pixels<'static>)>>,
    }

    impl<'a> ApplicationHandler for Loader<'a> {
      fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.result.is_none() {
          *self.result = Some(create_overlay_window(event_loop, self.config));
        }
      }
      fn window_event(&mut self, _el: &ActiveEventLoop, _id: WindowId, _ev: WindowEvent) {}
    }

    let mut loader = Loader {
      config,
      result: &mut window_pixels,
    };

    #[cfg(target_os = "windows")]
    {
      let _ = event_loop.pump_app_events(None, &mut loader);
    }

    #[cfg(not(target_os = "windows"))]
    {
      return Err(Error::new(
        Status::GenericFailure,
        "create_window_from_loop only supported on Windows in this build",
      ));
    }
  }

  if let Some(res) = window_pixels {
    res
  } else {
    Err(Error::new(
      Status::GenericFailure,
      "Failed to trigger window creation. Resume event was not received.",
    ))
  }
}

/// Create overlay window with optimized configuration
pub fn create_overlay_window(
  event_loop: &ActiveEventLoop,
  config: &WindowConfig,
) -> Result<(Arc<Window>, Pixels<'static>)> {
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

  let attributes = WindowAttributes::default()
    .with_transparent(transparent)
    .with_decorations(decorations)
    .with_title(&title)
    .with_resizable(resizable)
    .with_inner_size(LogicalSize::new(width, height));

  let attributes = match always_on_top {
    true => attributes.with_window_level(winit::window::WindowLevel::AlwaysOnTop),
    false => attributes.with_window_level(winit::window::WindowLevel::Normal),
  };

  // Set position if specified
  let attributes = if let (Some(x), Some(y)) = (config.x, config.y) {
    attributes.with_position(LogicalPosition::new(x, y))
  } else {
    attributes
  };

  // Set initial state
  let attributes = if config.fullscreen.unwrap_or(false) {
    attributes.with_fullscreen(Some(Fullscreen::Borderless(None)))
  } else {
    attributes
  };
  let attributes = if config.maximized.unwrap_or(false) {
    attributes.with_maximized(true)
  } else {
    attributes
  };
  let attributes = if config.minimized.unwrap_or(false) {
    attributes.with_visible(false)
  } else {
    attributes
  };

  let window = Arc::new(event_loop.create_window(attributes).map_err(|e| {
    Error::new(
      Status::GenericFailure,
      format!("Failed to create window: {}", e),
    )
  })?);

  // Get window size
  let window_size = window.inner_size();

  // Create pixels surface
  let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.clone());
  let mut pixels =
    Pixels::new(window_size.width, window_size.height, surface_texture).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Failed to create pixels: {}", e),
      )
    })?;

  // FIX: Set transparent clear color if requested
  if transparent {
    pixels.clear_color(pixels::wgpu::Color {
      r: 0.0,
      g: 0.0,
      b: 0.0,
      a: 0.0,
    });
  }

  // Unsafe cast to 'static because the window is owned by the same state
  let pixels_static = unsafe { std::mem::transmute::<Pixels<'_>, Pixels<'static>>(pixels) };

  Ok((window, pixels_static))
}

pub fn poll_event_loop(
  event_loop: &mut EventLoop<()>,
  windows: &[Arc<Mutex<WindowState>>],
) -> bool {
  let mut app = OverlayApplication {
    windows,
    exit_requested: false,
  };

  #[cfg(target_os = "windows")]
  {
    let _ = event_loop.pump_app_events(None, &mut app);
  }

  app.exit_requested
}

pub fn run_event_loop(event_loop: EventLoop<()>, windows: Vec<Arc<Mutex<WindowState>>>) -> ! {
  let mut app = OverlayApplication {
    windows: &windows,
    exit_requested: false,
  };

  event_loop
    .run_app(&mut app)
    .expect("Failed to run event loop");
  std::process::exit(0);
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
    let mut state = self.state.lock().unwrap();
    if let Some(window) = &state.window {
      let _ = window.request_inner_size(LogicalSize::new(width, height));
      state.width = width;
      state.height = height;
      if let Some(pixels) = &mut state.pixels {
        let _ = pixels.resize_surface(width, height);
        let _ = pixels.resize_buffer(width, height);
      }
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
      match level {
        WindowLevel::AlwaysOnTop => {
          window.set_window_level(winit::window::WindowLevel::AlwaysOnTop)
        }
        WindowLevel::AlwaysOnBottom => {
          window.set_window_level(winit::window::WindowLevel::AlwaysOnBottom)
        }
        WindowLevel::Normal => window.set_window_level(winit::window::WindowLevel::Normal),
      }
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

  pub fn set_render_when_occluded(&self, render: bool) {
    let mut state = self.state.lock().unwrap();
    state.render_when_occluded = render;
  }

  pub fn is_occluded(&self) -> bool {
    let state = self.state.lock().unwrap();
    state.occluded
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
    let state = self.state.lock().unwrap();
    if let Some(pixels) = &state.pixels {
      let frame = pixels.frame();
      if buffer_data.len() != frame.len() {
        return Err(Error::new(
          Status::GenericFailure,
          format!(
            "Buffer size mismatch: expected {}, got {}",
            frame.len(),
            buffer_data.len()
          ),
        ));
      }

      // Safe way to get mut frame since pixels is in Option
      drop(state);
      let mut state = self.state.lock().unwrap();
      let pixels = state.pixels.as_mut().unwrap();
      let frame = pixels.frame_mut();

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
    Ok(vec![state.width, state.height])
  }

  pub fn clear_frame(&self, color: &Color) -> Result<()> {
    let mut state = self.state.lock().unwrap();
    let frame_width = state.width;
    let frame_height = state.height;

    if let Some(pixels) = &mut state.pixels {
      let frame = pixels.frame_mut();
      crate::buffer::clear_buffer_optimized(frame, frame_width, frame_height, color);

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
    let mut state = self.state.lock().unwrap();

    let should_render = !state.occluded || state.render_when_occluded;

    if !should_render {
      return Ok(());
    }

    if let Some(pixels) = &mut state.pixels {
      pixels
        .render()
        .map_err(|e| Error::new(Status::GenericFailure, format!("Render error: {}", e)))?;
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
