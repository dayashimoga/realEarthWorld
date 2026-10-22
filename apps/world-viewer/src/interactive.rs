//! Interactive desktop windowing and keyboard/mouse navigation loop.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

use crate::camera_controller::{CameraController, CameraMode, Landmark};

pub struct InteractiveViewerApp {
    window: Option<Arc<Window>>,
    title: String,
    camera_mode: CameraMode,
    #[allow(dead_code)]
    is_drive: bool,
    #[allow(dead_code)]
    active_landmark: usize,
    pressed_keys: HashSet<KeyCode>,
    last_frame: Instant,
    camera: CameraController,
}

impl InteractiveViewerApp {
    pub fn new(title: &str, camera_mode: CameraMode, is_drive: bool) -> Self {
        let initial_landmark = Landmark::SAN_FRANCISCO_TRANSAMERICA;
        let mut camera = CameraController::new(initial_landmark.coord);
        camera.mode = camera_mode;

        Self {
            window: None,
            title: title.to_string(),
            camera_mode,
            is_drive,
            active_landmark: 0,
            pressed_keys: HashSet::new(),
            last_frame: Instant::now(),
            camera,
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
                    self.last_frame = Instant::now();
                    println!("🖥️ Native winit display window opened successfully: 1280x720");
                    println!("🎮 Controls: [W/S] Forward/Back, [A/D] Strafe, [Space/C] Up/Down, [Arrows] Rotate, [1-9] Landmarks, [Tab] Mode, [Esc] Exit");
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
                        state,
                        ..
                    },
                ..
            } => match state {
                ElementState::Pressed => {
                    self.pressed_keys.insert(key);
                    match key {
                        KeyCode::Escape => {
                            println!("Escape key pressed: exiting viewer.");
                            event_loop.exit();
                        }
                        KeyCode::Digit1 => {
                            self.camera.jump_to(&Landmark::SAN_FRANCISCO_TRANSAMERICA);
                            println!(
                                "Key 1: Jump to {}",
                                Landmark::SAN_FRANCISCO_TRANSAMERICA.name
                            );
                        }
                        KeyCode::Digit2 => {
                            self.camera.jump_to(&Landmark::SAN_FRANCISCO_SALESFORCE);
                            println!("Key 2: Jump to {}", Landmark::SAN_FRANCISCO_SALESFORCE.name);
                        }
                        KeyCode::Digit3 => {
                            self.camera.jump_to(&Landmark::SAN_FRANCISCO_FERRY_BUILDING);
                            println!(
                                "Key 3: Jump to {}",
                                Landmark::SAN_FRANCISCO_FERRY_BUILDING.name
                            );
                        }
                        KeyCode::Digit4 => {
                            self.camera.jump_to(&Landmark::SAN_FRANCISCO_COIT_TOWER);
                            println!("Key 4: Jump to {}", Landmark::SAN_FRANCISCO_COIT_TOWER.name);
                        }
                        KeyCode::Digit5 => {
                            self.camera.jump_to(&Landmark::SAN_FRANCISCO_BAY_BRIDGE);
                            println!("Key 5: Jump to {}", Landmark::SAN_FRANCISCO_BAY_BRIDGE.name);
                        }
                        KeyCode::Digit6 => {
                            self.camera.jump_to(&Landmark::BENGALURU_VIDHANA_SOUDHA);
                            println!("Key 6: Jump to {}", Landmark::BENGALURU_VIDHANA_SOUDHA.name);
                        }
                        KeyCode::Digit7 => {
                            self.camera.jump_to(&Landmark::MYSURU_PALACE);
                            println!("Key 7: Jump to {}", Landmark::MYSURU_PALACE.name);
                        }
                        KeyCode::Digit8 => {
                            self.camera.jump_to(&Landmark::CHENNAI_CENTRAL);
                            println!("Key 8: Jump to {}", Landmark::CHENNAI_CENTRAL.name);
                        }
                        KeyCode::Digit9 => {
                            self.camera.jump_to(&Landmark::HYDERABAD_CHARMINAR);
                            println!("Key 9: Jump to {}", Landmark::HYDERABAD_CHARMINAR.name);
                        }
                        KeyCode::Tab => {
                            self.camera_mode = match self.camera_mode {
                                CameraMode::Fly => CameraMode::Walk,
                                CameraMode::Walk => CameraMode::Fly,
                            };
                            self.camera.mode = self.camera_mode;
                            println!("Mode toggled to: {:?}", self.camera_mode);
                        }
                        _ => {}
                    }
                }
                ElementState::Released => {
                    self.pressed_keys.remove(&key);
                }
            },
            WindowEvent::RedrawRequested => {
                if let Some(ref window) = self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f64().min(0.1);
        self.last_frame = now;

        if !self.pressed_keys.is_empty() {
            let move_speed = if self.camera_mode == CameraMode::Fly {
                60.0 * dt
            } else {
                5.0 * dt
            };
            let rot_speed = 1.5 * dt as f32;

            if self.pressed_keys.contains(&KeyCode::KeyW) {
                self.camera.move_forward(move_speed, None);
            }
            if self.pressed_keys.contains(&KeyCode::KeyS) {
                self.camera.move_forward(-move_speed, None);
            }
            if self.pressed_keys.contains(&KeyCode::KeyA) {
                self.camera.strafe(-move_speed, None);
            }
            if self.pressed_keys.contains(&KeyCode::KeyD) {
                self.camera.strafe(move_speed, None);
            }
            if self.pressed_keys.contains(&KeyCode::Space) {
                self.camera.move_vertical(move_speed);
            }
            if self.pressed_keys.contains(&KeyCode::KeyC) {
                self.camera.move_vertical(-move_speed);
            }
            if self.pressed_keys.contains(&KeyCode::ArrowLeft) {
                self.camera.rotate(-rot_speed, 0.0);
            }
            if self.pressed_keys.contains(&KeyCode::ArrowRight) {
                self.camera.rotate(rot_speed, 0.0);
            }
            if self.pressed_keys.contains(&KeyCode::ArrowUp) {
                self.camera.rotate(0.0, rot_speed);
            }
            if self.pressed_keys.contains(&KeyCode::ArrowDown) {
                self.camera.rotate(0.0, -rot_speed);
            }
        }

        if let Some(ref window) = self.window {
            window.request_redraw();
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
