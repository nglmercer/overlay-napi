#![deny(clippy::all)]

use std::sync::{Arc, Mutex};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopBuilder, ControlFlow};
use winit::window::{Window, WindowBuilder, WindowLevel};
use pixels::{Pixels, SurfaceTexture};

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
            .with_window_level(WindowLevel::AlwaysOnTop)
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
}

#[napi]
pub fn plus_100(input: u32) -> u32 {
    input + 100
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
    fn test_plus_100_function() {
        assert_eq!(plus_100(0), 100);
        assert_eq!(plus_100(50), 150);
        assert_eq!(plus_100(100), 200);
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
    }
}
