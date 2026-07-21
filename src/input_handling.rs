use std::collections::HashMap;

use beryllium::events::SDL_Keycode;

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

pub struct InputHandler {
  keys: HashMap<SDL_Keycode, KeyData>,
  
}

impl InputHandler {
  
  pub fn new() -> InputHandler {
    InputHandler { keys: HashMap::with_capacity(104) }
  }

  pub fn get_key(&self, key: SDL_Keycode) -> KeyData {
    self.keys.get(&key).copied().unwrap_or(KeyData::none())
  }

  pub fn is_key_active(&self, key: SDL_Keycode) -> bool{
    let state = self.get_key(key).state;
    state == PressState::Down || state == PressState::Held 
  }

  pub fn activate_key(&mut self, key: SDL_Keycode) {
    let entry = self.keys.entry(key).or_insert(KeyData::none());
    
    if entry.state != PressState::Down && entry.state != PressState::Held {
      entry.state = PressState::Down;
    }
  }

  pub fn deactivate_key(&mut self, key: SDL_Keycode) {
    let entry = self.keys.entry(key).or_insert(KeyData::none());
    entry.state = PressState::Up;
  }

  pub fn update_key_state(&mut self) {
    for v in self.keys.values_mut()  {
      match v.state {
        PressState::Up => { v.state = PressState::None; },
        PressState::Down => { v.state = PressState::Held; },
        _ => {}
      }
    }
  }
}