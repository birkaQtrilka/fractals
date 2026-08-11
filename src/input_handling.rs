use std::collections::HashMap;

use beryllium::events::{Event, SDL_Keycode, SDLK_ESCAPE};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MouseButton {
  Left,
  Middle,
  Right,
  X1,
  X2,
  Other(u8),
}

#[derive(Copy, Clone, Debug)]
pub struct MouseEventData {
  pub x: i32,
  pub y: i32,
  pub button: Option<MouseButton>, // None for plain mouse_move
  pub delta_x: i32,
  pub delta_y: i32,
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
struct SubscriptionId(u64);

#[derive(Clone, Copy)]
enum MouseEventKind {
  Down,
  Up,
  Move,
}

#[derive(Clone, Copy)]
pub struct MouseSubscription {
  id: SubscriptionId,
  kind: MouseEventKind,
}

pub struct InputHandler {
  keys: HashMap<SDL_Keycode, KeyData>,

  last_mouse_x: i32,
  last_mouse_y: i32,

  next_sub_id: u64,
  mouse_down_subs: HashMap<SubscriptionId, MouseCallback>,
  mouse_up_subs: HashMap<SubscriptionId, MouseCallback>,
  mouse_move_subs: HashMap<SubscriptionId, MouseCallback>,
}

impl InputHandler {

  pub fn new() -> InputHandler {
    InputHandler {
      keys: HashMap::with_capacity(104),
      last_mouse_x: 0,
      last_mouse_y: 0,
      next_sub_id: 0,
      mouse_down_subs: HashMap::new(),
      mouse_up_subs: HashMap::new(),
      mouse_move_subs: HashMap::new(),
    }
  }

  pub fn main_loop(&mut self,) {
    self.update_key_state();
  }

  pub fn process_events(&mut self, event: Event) -> bool {
    match event {
      Event::Quit => return true,
      Event::Key { pressed, keycode, .. } => {
        if pressed {
          self.activate_key(keycode);
          // Check for escape key immediately
          if keycode == SDLK_ESCAPE {
            return true;
          }
        } else {
          self.deactivate_key(keycode);
        }
      }
      Event::MouseMotion { x_win, y_win, x_delta, y_delta, .. } => {
        self.fire_mouse_move(x_win, y_win, x_delta, y_delta);
      }
      Event::MouseButton { pressed, button, x, y, .. } => {
        let btn = match button {
          1 => MouseButton::Left,
          2 => MouseButton::Middle,
          3 => MouseButton::Right,
          4 => MouseButton::X1,
          5 => MouseButton::X2,
          other => MouseButton::Other(other),
        };

        if pressed {
          self.fire_mouse_down(x, y, btn);
        } else {
          self.fire_mouse_up(x, y, btn);
        }
      }
      _ => (),
    }
    return false;
  }

  pub fn get_key(&self, key: SDL_Keycode) -> KeyData {
    self.keys.get(&key).copied().unwrap_or(KeyData::none())
  }

  pub fn is_key_active(&self, key: SDL_Keycode) -> bool{
    let state = self.get_key(key).state;
    state == PressState::Down || state == PressState::Held 
  }

  pub fn is_key_down(&self, key: SDL_Keycode) -> bool {
    self.get_key(key).state == PressState::Down
  }

  fn activate_key(&mut self, key: SDL_Keycode) {
    let entry = self.keys.entry(key).or_insert(KeyData::none());

    if entry.state != PressState::Down && entry.state != PressState::Held {
      entry.state = PressState::Down;
    }
  }

  fn deactivate_key(&mut self, key: SDL_Keycode) {
    let entry = self.keys.entry(key).or_insert(KeyData::none());
    entry.state = PressState::Up;
  }

  fn update_key_state(&mut self) {
    for v in self.keys.values_mut()  {
      match v.state {
        PressState::Up => { v.state = PressState::None; },
        PressState::Down => { v.state = PressState::Held; },
        _ => {}
      }
    }
  }

  // MOUSE

  // events
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

  fn fire_mouse_down(&mut self, x: i32, y: i32, button: MouseButton) {
    let data = MouseEventData {
      x, y, button: Some(button),
      delta_x: x - self.last_mouse_x,
      delta_y: y - self.last_mouse_y,
    };
    self.last_mouse_x = x;
    self.last_mouse_y = y;
    for cb in self.mouse_down_subs.values_mut() {
      cb(&data);
    }
  }

  fn fire_mouse_up(&mut self, x: i32, y: i32, button: MouseButton) {
    let data = MouseEventData {
      x,
      y,
      button: Some(button),
      delta_x: x - self.last_mouse_x,
      delta_y: y - self.last_mouse_y,
    };
    self.last_mouse_x = x;
    self.last_mouse_y = y;

    for cb in self.mouse_up_subs.values_mut() {
      cb(&data);
    }
  }

  fn fire_mouse_move(&mut self, x: i32, y: i32, delta_x: i32, delta_y: i32) {
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