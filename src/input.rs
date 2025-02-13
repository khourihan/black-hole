use winit::{
    event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Clone)]
struct CurrentInput {
    mouse_actions: Vec<MouseAction>,
    key_actions: Vec<KeyAction>,
    keys_held: Vec<PhysicalKey>,
    mouse_held: [bool; 255],
    cursor_point: Option<(f32, f32)>,
    cursor_point_prev: Option<(f32, f32)>,
    mouse_diff: Option<(f32, f32)>,
    y_scroll_diff: f32,
    x_scroll_diff: f32,
}

impl CurrentInput {
    fn new() -> CurrentInput {
        CurrentInput {
            mouse_actions: vec![],
            key_actions: vec![],
            keys_held: vec![],
            mouse_held: [false; 255],
            cursor_point: None,
            cursor_point_prev: None,
            mouse_diff: None,
            y_scroll_diff: 0.0,
            x_scroll_diff: 0.0,
        }
    }

    fn step(&mut self) {
        self.mouse_actions.clear();
        self.key_actions.clear();
        self.cursor_point_prev = self.cursor_point;
        self.mouse_diff = None;
        self.y_scroll_diff = 0.0;
        self.x_scroll_diff = 0.0;
    }

    fn handle_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => match event.state {
                ElementState::Pressed => {
                    let physical_key = &event.physical_key;
                    if !self.keys_held.contains(physical_key) {
                        self.key_actions.push(KeyAction::Pressed(*physical_key));
                        self.keys_held.push(*physical_key);
                    }

                    self.key_actions.push(KeyAction::PressedOs(*physical_key));
                },
                ElementState::Released => {
                    let physical_key = &event.physical_key;
                    self.keys_held.retain(|x| x != physical_key);
                    self.key_actions.push(KeyAction::Released(*physical_key));
                },
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_point = Some((position.x as f32, position.y as f32));
            },
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                let button_usize = hash_mouse_button(button);
                self.mouse_held[button_usize] = true;
                self.mouse_actions.push(MouseAction::Pressed(*button));
            },
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button,
                ..
            } => {
                let button_usize = hash_mouse_button(button);
                self.mouse_held[button_usize] = false;
                self.mouse_actions.push(MouseAction::Released(*button));
            },
            WindowEvent::MouseWheel { delta, .. } => {
                const PIXELS_PER_LINE: f64 = 38.0;

                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.x_scroll_diff += x;
                        self.y_scroll_diff += y;
                    },
                    MouseScrollDelta::PixelDelta(delta) => {
                        self.y_scroll_diff += (delta.y / PIXELS_PER_LINE) as f32;
                        self.x_scroll_diff += (delta.x / PIXELS_PER_LINE) as f32;
                    },
                }
            },
            _ => {},
        }
    }

    fn handle_device_event(&mut self, event: &DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta, .. } = event {
            match self.mouse_diff {
                Some((x, y)) => self.mouse_diff = Some((x + delta.0 as f32, y + delta.1 as f32)),
                None => self.mouse_diff = Some((delta.0 as f32, delta.1 as f32)),
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum KeyAction {
    Pressed(PhysicalKey),
    PressedOs(PhysicalKey),
    Released(PhysicalKey),
}

#[derive(Clone)]
pub enum MouseAction {
    Pressed(MouseButton),
    Released(MouseButton),
}

pub fn hash_mouse_button(button: &MouseButton) -> usize {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
        MouseButton::Other(byte) => 5 + *byte as usize,
    }
}

#[derive(Clone)]
pub struct InputManager {
    current: Option<CurrentInput>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    pub fn new() -> InputManager {
        InputManager {
            current: Some(CurrentInput::new()),
        }
    }

    pub fn step(&mut self) {
        if let Some(current) = &mut self.current {
            current.step();
        }
    }

    pub fn process_window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Focused(false) => self.current = None,
            WindowEvent::Focused(true) => {
                if self.current.is_none() {
                    self.current = Some(CurrentInput::new())
                }
            },
            _ => {},
        }
        if let Some(current) = &mut self.current {
            current.handle_window_event(event);
        }
    }

    pub fn process_device_event(&mut self, event: &DeviceEvent) {
        if let Some(ref mut current) = self.current {
            current.handle_device_event(event);
        }
    }

    pub fn key_pressed(&self, keycode: KeyCode) -> bool {
        let key = PhysicalKey::Code(keycode);
        if let Some(current) = &self.current {
            let searched_action = KeyAction::Pressed(key);
            if current.key_actions.contains(&searched_action) {
                return true;
            }
        }
        false
    }

    pub fn mouse_held(&self, mouse_button: MouseButton) -> bool {
        match &self.current {
            Some(current) => current.mouse_held[hash_mouse_button(&mouse_button)],
            None => false,
        }
    }

    pub fn scroll_diff(&self) -> (f32, f32) {
        match &self.current {
            Some(current) => (current.x_scroll_diff, current.y_scroll_diff),
            None => (0.0, 0.0),
        }
    }

    pub fn cursor(&self) -> Option<(f32, f32)> {
        match &self.current {
            Some(current) => current.cursor_point,
            None => None,
        }
    }

    pub fn mouse_diff(&self) -> (f32, f32) {
        if let Some(current_input) = &self.current {
            if let Some(diff) = current_input.mouse_diff {
                return diff;
            }
        }
        (0.0, 0.0)
    }
}
