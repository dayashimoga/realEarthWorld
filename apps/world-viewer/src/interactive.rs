//! Interactive desktop windowing and keyboard/mouse navigation loop.

use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::camera_controller::CameraMode;

pub struct InteractiveViewerApp {
    window: Option<Arc<Window>>,
    title: String,
    camera_mode: CameraMode,
    #[allow(dead_code)]
    is_drive: bool,
    #[allow(dead_code)]
    active_landmark: usize,
}

impl InteractiveViewerApp {
    pub fn new(title: &str, camera_mode: CameraMode, is_drive: bool) -> Self {
        Self {
            window: None,
            title: title.to_string(),
            camera_mode,
            is_drive,
            active_landmark: 0,
        }
    }
}

impl ApplicationHandler for InteractiveViewerApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = WindowAttributes::default()
                .with_title(&self.title)
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
            match event_loop.create_window(attrs) {
                Ok(w) => {
                    self.window = Some(Arc::new(w));
                    println!("🖥️ Native winit display window opened successfully: 1280x720");
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to create display window: {}", e);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("👋 Interactive window close requested. Exiting cleanly.");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key {
                KeyCode::Escape => {
                    println!("Escape key pressed: exiting viewer.");
                    event_loop.exit();
                }
                KeyCode::Digit1 => {
                    println!("Key 1: Jump to San Francisco Transamerica Pyramid");
                }
                KeyCode::Digit2 => {
                    println!("Key 2: Jump to New York One World Trade Center");
                }
                KeyCode::Digit3 => {
                    println!("Key 3: Jump to Los Angeles Wilshire Grand");
                }
                KeyCode::Digit4 => {
                    println!("Key 4: Jump to Chicago Willis Tower");
                }
                KeyCode::Digit5 => {
                    println!("Key 5: Jump to Seattle Space Needle");
                }
                KeyCode::Digit6 => {
                    println!("Key 6: Jump to Miami Brickell");
                }
                KeyCode::Digit7 => {
                    println!("Key 7: Jump to Bengaluru Vidhana Soudha");
                }
                KeyCode::Digit8 => {
                    println!("Key 8: Jump to Mysuru Palace");
                }
                KeyCode::Digit9 => {
                    println!("Key 9: Jump to Chennai Central");
                }
                KeyCode::Tab => {
                    self.camera_mode = match self.camera_mode {
                        CameraMode::Fly => CameraMode::Walk,
                        CameraMode::Walk => CameraMode::Fly,
                    };
                    println!("Mode toggled to: {:?}", self.camera_mode);
                }
                _ => {}
            },
            WindowEvent::RedrawRequested => {
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

/// Runs the interactive window loop.
pub fn run_interactive_event_loop(
    title: &str,
    camera_mode: CameraMode,
    is_drive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = InteractiveViewerApp::new(title, camera_mode, is_drive);
    event_loop.run_app(&mut app)?;
    Ok(())
}
