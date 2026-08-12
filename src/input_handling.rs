use std::collections::HashMap;

use winit::event::{ElementState, MouseButton as WinitMouseButton, WindowEvent, KeyEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MouseButton {
  Left,
  Middle,
  Right,
  Back,
  Forward,
  Other(u16),
}

impl From<WinitMouseButton> for MouseButton {
  fn from(btn: WinitMouseButton) -> Self {
    match btn {
      WinitMouseButton::Left => MouseButton::Left,
      WinitMouseButton::Middle => MouseButton::Middle,
      WinitMouseButton::Right => MouseButton::Right,
      WinitMouseButton::Back => MouseButton::Back,
      WinitMouseButton::Forward => MouseButton::Forward,
      WinitMouseButton::Other(x) => MouseButton::Other(x),
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub struct MouseEventData {
  pub x: f32,
  pub y: f32,
  pub button: Option<MouseButton>, // None for plain mouse_move
  pub delta_x: f32,
  pub delta_y: f32,
}

type MouseCallback = Box<dyn FnMut(&MouseEventData)>;

#[derive(Copy, Clone)] 
pub struct KeyData {
  pub state: PressState
}

impl KeyData {
  fn none() -> KeyData {
    KeyData {
      state: PressState::None
    }
  }
}

#[derive(Copy, Clone, PartialEq)] 
pub enum PressState {
  None,
  Down,
  Held,
  Up,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(u64);

#[derive(Clone, Copy)]
pub enum MouseEventKind {
  Down,
  Up,
  Move,
}

#[derive(Clone, Copy)]
pub struct MouseSubscription {
  pub id: SubscriptionId,
  pub kind: MouseEventKind,
}

pub struct InputHandler {
  keys: HashMap<KeyCode, KeyData>,

  last_mouse_x: f32,
  last_mouse_y: f32,

  next_sub_id: u64,
  mouse_down_subs: HashMap<SubscriptionId, MouseCallback>,
  mouse_up_subs: HashMap<SubscriptionId, MouseCallback>,
  mouse_move_subs: HashMap<SubscriptionId, MouseCallback>,
}

impl InputHandler {
  pub fn new() -> InputHandler {
    InputHandler {
      keys: HashMap::with_capacity(104),
      last_mouse_x: 0.0,
      last_mouse_y: 0.0,
      next_sub_id: 0,
      mouse_down_subs: HashMap::new(),
      mouse_up_subs: HashMap::new(),
      mouse_move_subs: HashMap::new(),
    }
  }

  pub fn main_loop(&mut self) {
    self.update_key_state();
  }

  pub fn process_events(&mut self, event: &WindowEvent) -> bool {
    match event {
      WindowEvent::KeyboardInput {
        event: KeyEvent { physical_key, state, .. },
        ..
      } => {
        if let PhysicalKey::Code(keycode) = physical_key {
          if *state == ElementState::Pressed {
            self.activate_key(*keycode);
            // Check for escape key immediately
            if *keycode == KeyCode::Escape {
              return true;
            }
          } else {
            self.deactivate_key(*keycode);
          }
        }
      }
      WindowEvent::CursorMoved { position, .. } => {
        let x = position.x as f32;
        let y = position.y as f32;
        let delta_x = x - self.last_mouse_x;
        let delta_y = y - self.last_mouse_y;
        self.fire_mouse_move(x, y, delta_x, delta_y);
      }
      WindowEvent::MouseInput { state, button, .. } => {
        let btn = MouseButton::from(*button);
        if *state == ElementState::Pressed {
          self.fire_mouse_down(self.last_mouse_x, self.last_mouse_y, btn);
        } else {
          self.fire_mouse_up(self.last_mouse_x, self.last_mouse_y, btn);
        }
      }
      _ => {}
    }
    return false;
  }

  pub fn get_key(&self, key: KeyCode) -> KeyData {
    self.keys.get(&key).copied().unwrap_or(KeyData::none())
  }

  pub fn is_key_active(&self, key: KeyCode) -> bool {
    let state = self.get_key(key).state;
    state == PressState::Down || state == PressState::Held 
  }

  pub fn is_key_down(&self, key: KeyCode) -> bool {
    self.get_key(key).state == PressState::Down
  }

  fn activate_key(&mut self, key: KeyCode) {
    let entry = self.keys.entry(key).or_insert(KeyData::none());

    if entry.state != PressState::Down && entry.state != PressState::Held {
      entry.state = PressState::Down;
    }
  }

  fn deactivate_key(&mut self, key: KeyCode) {
    let entry = self.keys.entry(key).or_insert(KeyData::none());
    entry.state = PressState::Up;
  }

  pub fn update_key_state(&mut self) {
    for v in self.keys.values_mut() {
      match v.state {
        PressState::Up => { v.state = PressState::None; },
        PressState::Down => { v.state = PressState::Held; },
        _ => {}
      }
    }
  }

  // MOUSE

  fn next_id(&mut self) -> SubscriptionId {
    let id = SubscriptionId(self.next_sub_id);
    self.next_sub_id += 1;
    id
  }

  pub fn on_mouse_down<F>(&mut self, callback: F) -> MouseSubscription
  where
    F: FnMut(&MouseEventData) + 'static,
  {
    let id = self.next_id();
    self.mouse_down_subs.insert(id, Box::new(callback));
    MouseSubscription { id, kind: MouseEventKind::Down }
  }

  pub fn on_mouse_up<F>(&mut self, callback: F) -> MouseSubscription
  where
    F: FnMut(&MouseEventData) + 'static,
  {
    let id = self.next_id();
    self.mouse_up_subs.insert(id, Box::new(callback));
    MouseSubscription { id, kind: MouseEventKind::Up }
  }

  pub fn on_mouse_move<F>(&mut self, callback: F) -> MouseSubscription
  where
    F: FnMut(&MouseEventData) + 'static,
  {
    let id = self.next_id();
    self.mouse_move_subs.insert(id, Box::new(callback));
    MouseSubscription { id, kind: MouseEventKind::Move }
  }

  pub fn unsubscribe_mouse(&mut self, sub: MouseSubscription) {
    match sub.kind {
      MouseEventKind::Down => { self.mouse_down_subs.remove(&sub.id); }
      MouseEventKind::Up => { self.mouse_up_subs.remove(&sub.id); }
      MouseEventKind::Move => { self.mouse_move_subs.remove(&sub.id); }
    }
  }

  fn fire_mouse_down(&mut self, x: f32, y: f32, button: MouseButton) {
    let data = MouseEventData {
      x,
      y,
      button: Some(button),
      delta_x: 0.0,
      delta_y: 0.0,
    };
    for cb in self.mouse_down_subs.values_mut() {
      cb(&data);
    }
  }

  fn fire_mouse_up(&mut self, x: f32, y: f32, button: MouseButton) {
    let data = MouseEventData {
      x,
      y,
      button: Some(button),
      delta_x: 0.0,
      delta_y: 0.0,
    };
    for cb in self.mouse_up_subs.values_mut() {
      cb(&data);
    }
  }

  fn fire_mouse_move(&mut self, x: f32, y: f32, delta_x: f32, delta_y: f32) {
    let data = MouseEventData {
      x,
      y,
      button: None,
      delta_x,
      delta_y,
    };
    self.last_mouse_x = x;
    self.last_mouse_y = y;

    for cb in self.mouse_move_subs.values_mut() {
      cb(&data);
    }
  }
}