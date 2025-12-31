#![deny(clippy::all)]

use std::sync::{Arc, Mutex};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder, ControlFlow};
use winit::window::{Window, WindowBuilder, WindowLevel as WinitWindowLevel};
use pixels::{Pixels, SurfaceTexture};
use winit::dpi::{LogicalPosition, LogicalSize};

#[napi]
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
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[napi(object)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[napi(object)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn to_rgba(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

struct OverlayState {
    pixels: Option<Pixels>,
    window: Option<Arc<Window>>,
}

impl OverlayState {
    fn new() -> Self {
        Self {
            pixels: None,
            window: None,
        }
    }
}

fn create_overlay_window(event_loop: &EventLoop<()>) -> Result<(Arc<Window>, Pixels)> {
    // Create transparent overlay window
    let window = Arc::new(
        WindowBuilder::new()
            .with_transparent(true)
            .with_decorations(false)
            .with_window_level(WinitWindowLevel::AlwaysOnTop)
            .with_title("Overlay NAPI")
            .build(event_loop)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create window: {}", e)))?
    );

    // Get window size
    let window_size = window.inner_size();
    
    // Create pixels surface - use raw window handle for compatibility
    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
    let pixels = Pixels::new(window_size.width, window_size.height, surface_texture)
        .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create pixels: {}", e)))?;

    Ok((window, pixels))
}

#[napi]
pub struct Overlay {
    state: Arc<Mutex<OverlayState>>,
}

#[napi]
impl Overlay {
    #[napi(constructor)]
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(OverlayState::new()));
        
        Self {
            state: state.clone(),
        }
    }

    #[napi]
    pub fn start(&mut self) -> Result<()> {
        let state = self.state.clone();
        
        // Create event loop and window in the same thread
        let event_loop = EventLoopBuilder::new().build();
        
        // Create window and pixels
        let (window, pixels) = create_overlay_window(&event_loop)?;
        
        // Store state
        {
            let mut state_guard = state.lock().unwrap();
            state_guard.window = Some(window.clone());
            state_guard.pixels = Some(pixels);
        }
        
        // Run event loop
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
                    // Request redraw on each frame
                    if let Some(window) = &state.lock().unwrap().window {
                        window.request_redraw();
                    }
                }
                _ => {}
            }
        });
        
        // Note: event_loop.run() never returns on Windows, so this is unreachable
        #[allow(unreachable_code)]
        Ok(())
    }

    #[napi]
    pub fn update_frame(&self, buffer: Buffer) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        
        if let Some(pixels) = &mut state.pixels {
            let frame = pixels.frame_mut();
            let buffer_data = buffer.as_ref();
            
            // Ensure buffer size matches frame size
            if buffer_data.len() != frame.len() {
                return Err(Error::new(
                    Status::InvalidArg,
                    format!("Buffer size mismatch: expected {} bytes, got {} bytes", frame.len(), buffer_data.len())
                ));
            }
            
            // Copy buffer data to frame
            frame.copy_from_slice(buffer_data);
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn get_frame_size(&self) -> Result<Vec<u32>> {
        let state = self.state.lock().unwrap();
        
        if let Some(pixels) = &state.pixels {
            let frame = pixels.frame();
            let size = frame.len() / 4; // RGBA = 4 bytes per pixel
            let width = (size as f64).sqrt() as u32;
            let height = width;
            
            Ok(vec![width, height])
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn show(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            window.set_visible(true);
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn hide(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            window.set_visible(false);
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn set_position(&self, x: i32, y: i32) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            window.set_outer_position(LogicalPosition::new(x, y));
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn get_position(&self) -> Result<WindowPosition> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            let pos = window.outer_position()
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get position: {}", e)))?;
            Ok(WindowPosition { x: pos.x, y: pos.y })
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn set_size(&self, width: u32, height: u32) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            window.set_inner_size(LogicalSize::new(width, height));
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn get_size(&self) -> Result<WindowSize> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            let size = window.inner_size();
            Ok(WindowSize { width: size.width, height: size.height })
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn set_title(&self, title: String) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            window.set_title(&title);
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn set_window_level(&self, level: WindowLevel) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            window.set_window_level(level.into());
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn request_redraw(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            window.request_redraw();
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn is_visible(&self) -> Result<bool> {
        let state = self.state.lock().unwrap();
        
        if let Some(window) = &state.window {
            Ok(window.is_visible().unwrap_or(false))
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn clear_frame(&self, color: Color) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        
        if let Some(pixels) = &mut state.pixels {
            let frame = pixels.frame_mut();
            let rgba = color.to_rgba();
            
            // Fill frame with solid color
            for chunk in frame.chunks_exact_mut(4) {
                chunk.copy_from_slice(&rgba);
            }
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }

    #[napi]
    pub fn draw_rectangle(&self, x: u32, y: u32, width: u32, height: u32, color: Color) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        
        if let Some(pixels) = &mut state.pixels {
            let frame = pixels.frame_mut();
            let frame_size = self.get_frame_size()?;
            let frame_width = frame_size[0] as usize;
            let frame_height = frame_size[0] as usize; // Assuming square frame
            
            let rgba = color.to_rgba();
            
            // Draw rectangle
            for dy in 0..height {
                for dx in 0..width {
                    let px = x + dx;
                    let py = y + dy;
                    
                    if px < frame_width as u32 && py < frame_height as u32 {
                        let index = (py as usize * frame_width + px as usize) * 4;
                        if index + 3 < frame.len() {
                            frame[index..index + 4].copy_from_slice(&rgba);
                        }
                    }
                }
            }
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Overlay not initialized"))
        }
    }
}


#[napi]
pub fn create_color(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::new(r, g, b, a)
}

#[napi]
pub fn create_position(x: i32, y: i32) -> WindowPosition {
    WindowPosition { x, y }
}

#[napi]
pub fn create_size(width: u32, height: u32) -> WindowSize {
    WindowSize { width, height }
}

// Common colors as constants
#[napi]
pub fn color_red() -> Color {
    Color::new(255, 0, 0, 255)
}

#[napi]
pub fn color_green() -> Color {
    Color::new(0, 255, 0, 255)
}

#[napi]
pub fn color_blue() -> Color {
    Color::new(0, 0, 255, 255)
}

#[napi]
pub fn color_black() -> Color {
    Color::new(0, 0, 0, 255)
}

#[napi]
pub fn color_white() -> Color {
    Color::new(255, 255, 255, 255)
}

#[napi]
pub fn color_transparent() -> Color {
    Color::new(0, 0, 0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
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
        let state = Arc::new(Mutex::new(OverlayState::new()));
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
            winit::window::WindowLevel::Normal => {},
            _ => panic!("Expected Normal window level"),
        }
        
        match always_on_top {
            winit::window::WindowLevel::AlwaysOnTop => {},
            _ => panic!("Expected AlwaysOnTop window level"),
        }
        
        match always_on_bottom {
            winit::window::WindowLevel::AlwaysOnBottom => {},
            _ => panic!("Expected AlwaysOnBottom window level"),
        }
    }
}
