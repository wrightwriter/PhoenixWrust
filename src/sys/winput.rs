use std::{collections::HashMap};

use nalgebra_glm::Vec2;
use smallvec::SmallVec;
use winit::{
  dpi::PhysicalPosition,
  event::{ElementState, MouseButton, VirtualKeyCode},
};

pub type WKeyCode = VirtualKeyCode;

#[derive(Default, Clone, Copy)]
pub struct KeyState {
  pub pressed: bool,
  pub down: bool,
  pub released: bool,
}

// impl Default for KeyState{
//     fn default() -> Self {
//         Self {
//             pressed: false,
//             down: false,
//             unpressed: todo!()
//         }
//     }
// }

#[derive(Default, Clone, Copy)]
pub struct MouseState {
  pub lmb_down: bool,
  pub lmb_press: bool,
  pub lmb_release: bool,
  pub rmb_down: bool,
  pub rmb_press: bool,
  pub rmb_release: bool,
  pub pos: nalgebra_glm::Vec2,
  pub delta_pos: nalgebra_glm::Vec2,


  pub pos_normalized: nalgebra_glm::Vec2,
  pub delta_pos_normalized: nalgebra_glm::Vec2,
}

pub struct WInput {
  pub key_states: HashMap<WKeyCode, KeyState>,

  pub mouse_state: MouseState,

  pub window_focused: bool,

  pub keys_to_unrelease: SmallVec<[WKeyCode; 10]>,
  pub keys_to_unpress: SmallVec<[WKeyCode; 10]>,
}

impl WInput {
  pub fn handle_key_press(
    &mut self,
    keycode: WKeyCode,
    state: ElementState,
  ) {
    unsafe {
      let mut key_state = self.get_key(keycode);

      match state {
        ElementState::Pressed => {
          if key_state.down == false {
            key_state.pressed = true;
            self.keys_to_unpress.push(keycode);
          }
          key_state.down = true;
        }
        ElementState::Released => {
          key_state.released = true;
          self.keys_to_unrelease.push(keycode);
          key_state.down = false;
        }
      }
      self.key_states.insert(keycode, key_state);
      // (VirtualKeyCode::Escape, ElementState::Released) => *control_flow = ControlFlow::Exit,
    }
  }

  pub fn handle_mouse_move(
    &mut self,
    position: PhysicalPosition<f64>,
    width: f32,
    height: f32,
  ) {
    unsafe {
      let mut mouse_state = &mut self.mouse_state;

      let new_mouse_pos: Vec2 = Vec2::new(position.x as f32, position.y as f32);

      let delta_pos: Vec2 = new_mouse_pos - mouse_state.pos.clone();

      mouse_state.delta_pos = delta_pos;
      mouse_state.pos = new_mouse_pos;

      // useless loc
      let screen_res: Vec2 = Vec2::new(width, height);

      mouse_state.pos_normalized = mouse_state.pos;
      mouse_state.pos_normalized[0] /= screen_res.x;
      mouse_state.pos_normalized[1] /= screen_res.x;

      mouse_state.delta_pos_normalized = delta_pos;
      mouse_state.delta_pos_normalized[0] /= screen_res.x;
      mouse_state.delta_pos_normalized[1] /= screen_res.x;

      // (VirtualKeyCode::Escape, ElementState::Released) => *control_flow = ControlFlow::Exit,
    }
  }
  pub fn handle_mouse_press(
    &mut self,
    button: MouseButton,
    state: ElementState,
  ) {
    unsafe {
      let mut mouse_state = &mut self.mouse_state;

      match button {
        winit::event::MouseButton::Left => match state {
          ElementState::Pressed => {
            mouse_state.lmb_press = true;
            mouse_state.lmb_down = true;
          }
          ElementState::Released => {
            mouse_state.lmb_release = true;
            mouse_state.lmb_down = false;
          }
        },
        winit::event::MouseButton::Right => match state {
          ElementState::Pressed => {
            mouse_state.rmb_press = true;
            mouse_state.rmb_down = true;
          }
          ElementState::Released => {
            mouse_state.rmb_release = true;
            mouse_state.rmb_down = false;
          }
        },
        winit::event::MouseButton::Middle => {}
        winit::event::MouseButton::Other(_) => {}
      }
      // mouse_state.pes

      // (VirtualKeyCode::Escape, ElementState::Released) => *control_flow = ControlFlow::Exit,
    }
  }
  pub fn refresh_keys(&mut self) {
    for key_code in &self.keys_to_unrelease {
      let key_state = self.key_states.get_mut(key_code).unwrap();
      key_state.released = false;
    }

    for key_code in &self.keys_to_unpress {
      let key_state = self.key_states.get_mut(key_code).unwrap();
      key_state.pressed = false;
    }
    unsafe {
      self.keys_to_unrelease.set_len(0);
      self.keys_to_unpress.set_len(0);
    }
    self.mouse_state.lmb_press = false;
    self.mouse_state.rmb_press = false;
    self.mouse_state.lmb_release = false;
    self.mouse_state.rmb_release = false;
  }
  pub fn get_key(
    &self,
    key_code: WKeyCode,
  ) -> KeyState {
    match self.key_states.get(&key_code) {
      Some(key_state) => *key_state,
      None => KeyState { ..wdef!() },
    }
  }
  pub fn new() -> Self {
    let mut key_states = HashMap::new();
    let mouse_state = MouseState { ..wdef!() };
    let keys_to_unrelease = SmallVec::new();
    let keys_to_unpress = SmallVec::new();

    Self {
      key_states,
      mouse_state,
      keys_to_unrelease,
      keys_to_unpress,
      window_focused: true,
    }
  }
}
