use std::{f32::consts::PI};

use nalgebra_glm::{
  cos, cross, length, look_at, normalize, radians, sin, vec1, vec2, vec3, Mat4, Vec3,
};

use crate::sys::winput::{MouseState, WInput, WKeyCode};

pub enum WCameraProjection {
  Perspective,
  Orthographic,
}

// pub enum WCameraType{
//     Pilot,
//     Static,
// }

pub struct WCamera {
  pub up: Vec3,
  pub eye_pos: Vec3,
  pub target_pos: Vec3,

  pub forward_vec: Vec3,
  pub pitch: f32,
  pub yaw: f32,

  pub view_mat: Mat4,
  pub proj_mat: Mat4,
  pub view_proj_mat: Mat4,
  pub inv_view_mat: Mat4,
  pub inv_proj_mat: Mat4,

  pub prev_view_mat: Mat4,
  pub prev_proj_mat: Mat4,
  pub prev_view_proj_mat: Mat4,
  pub prev_inv_view_mat: Mat4,
  pub prev_inv_proj_mat: Mat4,

  pub width: u32,
  pub height: u32,
  pub aspect_ratio: f32,

  pub fov: f32,
  pub near: f32,
  pub far: f32,

  pub cam_speed: f32,

  pub piloted: bool,
  pub cam_projection: WCameraProjection,
}

impl WCamera {
  fn change_cam_proj(
    &mut self,
    cam_proj: WCameraProjection,
  ) {
    self.cam_projection = cam_proj;
  }

  fn update_aspect_ratio() {
    todo!();
  }
  pub fn new(
    width: u32,
    height: u32,
  ) -> Self {
    let eye_pos = Vec3::new(0.0, 3.7, -1.0);
    let target_pos = eye_pos + vec3(0.0,0.0,1.0);
    let view_mat = Mat4::identity();
    let proj_mat = Mat4::identity();
    let inv_view_mat = Mat4::identity();
    let inv_proj_mat = Mat4::identity();
    let up = Vec3::new(0.0, 1.0, 0.0);

    let aspect_ratio = width as f32 / height as f32;

    Self {
      eye_pos,
      target_pos,
      view_mat,
      proj_mat,
      inv_view_mat,
      inv_proj_mat,
      up,
      width,
      height,
      aspect_ratio,
      fov: 90.0,
      near: 0.01,
      far: 100.0,
      cam_speed: 0.5,
      cam_projection: WCameraProjection::Perspective,
      piloted: true,
      forward_vec: Vec3::new(0.0, 0.0, 1.0),
      pitch: 0.0,
      yaw: 0.0,
      view_proj_mat: Mat4::identity(),
    prev_view_mat: Mat4::identity(),
    prev_proj_mat: Mat4::identity(),
    prev_view_proj_mat: Mat4::identity(),
    prev_inv_view_mat: Mat4::identity(),
    prev_inv_proj_mat: Mat4::identity(),
    }
  }

  pub fn update_movement(
    &mut self,
    mouse_delta: MouseState,
    w_input: &WInput,
    delta_time: f32,
  ) {
    if !self.piloted || !w_input.mouse_state.rmb_down {
        return;
    }
    enum Axis {
      X,
      Y,
      Z,
    }

    fn wcos(rad: f32) -> f32 {
      cos(&vec1(rad))[0]
    }

    fn wsin(rad: f32) -> f32 {
      sin(&vec1(rad))[0]
    }

    fn rotate(
      v: Vec3,
      axis: Axis,
      rad: f32,
    ) -> Vec3 {
      let mut ret = v.clone();

      let mut ax_a = 0;
      let mut ax_b = 0;

      match axis {
        Axis::X => {
          ax_a = 1;
          ax_b = 2;
        }
        Axis::Y => {
          ax_a = 0;
          ax_b = 2;
        }
        Axis::Z => {
          ax_a = 0;
          ax_b = 1;
        }
      }

      ret[ax_a] = wcos(rad) * v[ax_a] - wsin(rad) * v[ax_b];
      ret[ax_b] = wsin(rad) * v[ax_a] + wcos(rad) * v[ax_b];

      return ret;
    }

    let pi = PI;
    let tau = PI * 2.0;
    // let mut dir = Vec3::new(0.0,0.0,1.0);
    let mut dir = vec3(0.0, 0.0, 1.0);

    self.pitch += mouse_delta.delta_pos_normalized.x;
    self.yaw += mouse_delta.delta_pos_normalized.y;
    

    // if self.yaw > tau {
    //     self.yaw = tau;
    // }
    
    let miin = -0.234;
    let maax = 0.23;

    if self.yaw < miin {
        self.yaw = miin;
    }

    if self.yaw > maax {
        self.yaw = maax;
    }
    

    dir = rotate(dir, Axis::X,self.yaw* tau);
    dir = rotate(dir, Axis::Y, -self.pitch * tau);

    let len = length(&dir);

    let right = normalize(&cross(&self.up, &dir));
    let up = cross(&dir, &right);

    
    let local_speed = 1f32;
        // walk
    // if w_input.get_key(WKeyCode::LControl).down {

      let key_input_roll = 0.0f32;
      let mut local_speed = self.cam_speed * 5.0 * delta_time;

      let mut keyInput = vec2(0.0, 0.0);

      if w_input.get_key(WKeyCode::A).down {

          keyInput[0] -= 1.0;
      }
      if w_input.get_key(WKeyCode::D).down {

          keyInput[0] += 1.0;
      }
      if w_input.get_key(WKeyCode::W).down {

          keyInput[1] += 1.0;
      }
      if w_input.get_key(WKeyCode::S).down {

          keyInput[1] -= 1.0;
      }
    //   if (w_input.get_key(WKeyCode::).down) {
    //       keyInputRoll -= 1.0f;
    //   }
    //   if (w_input.get_key(WKeyCode::).down) {
    //       keyInputRoll += 1.0f;
    //   }
      if w_input.get_key(WKeyCode::LShift).down {
          local_speed *= 2.0;
      }

      let delta_dir = dir * keyInput[1] * local_speed;
      let delta_right = right * keyInput[0] * local_speed;

      self.eye_pos += delta_right + delta_dir;

    //   roll += keyInputRoll * localSpeed * 0.1f
    // }
    
    self.target_pos = self.eye_pos + dir;

      //             val keyInput = Vec2(0.0f)
      //             if (glfwGetWindowAttrib(Global.engine.glfwWindow, GLFW_FOCUSED) == GLFW_TRUE) {
      //                 if (io.keyboard[IO.Key.A]!!.Down) {
      //                     keyInput[0] -= 1.0f;
      //                 }
      //                 if (io.keyboard[IO.Key.D]!!.Down) {
      //                     keyInput[0] += 1.0f;
      //                 }
      //                 if (io.keyboard[IO.Key.W]!!.Down) {
      //                     keyInput[1] += 1.0f;
      //                 }
      //                 if (io.keyboard[IO.Key.S]!!.Down) {
      //                     keyInput[1] -= 1.0f;
      //                 }
      //                 if (io.keyboard[IO.Key.Q]!!.Down) {
      //                     keyInputRoll -= 1.0f;
      //                 }
      //                 if (io.keyboard[IO.Key.E]!!.Down) {
      //                     keyInputRoll += 1.0f;
      //                 }
      //                 if (io.keyboard[IO.Key.LShift]!!.Down) {
      //                     localSpeed *= 2.0f;
      //                 }
      // //                if (keyInput[0] == 0 && keyInput[1] == 0 && keyInputRoll == 0) {
      // //                    return false;
      // //                }


      //                 //float* up = Math::normalize(Math::cross(dir, right));
      //                 //up = Math::multiply(up, keyInput[0]);

    //         val right = Vec3.normalize(Vec3.cross(Vec3(0,1,0), dir))
    //         val up: Vec3 = Vec3.cross(dir, right)

    //         val dir = Vec3(0, 0, 1)
    //             .rotX (pitch * MathUtils.Tau)
    //             .rotY( yaw * MathUtils.Tau)

    //         val len = Vec3.length(dir)

    //         val right = Vec3.normalize(Vec3.cross(Vec3(0,1,0), dir))
    //         val up: Vec3 = Vec3.cross(dir, right)

    //         if (io.keyboard[IO.Key.LCtrl]!!.Down){
    //             var keyInputRoll = 0.0f;
    //             var localSpeed = speed * 5.0f*Global.engine.deltaTime;

    //             val keyInput = Vec2(0.0f)
    //             if (glfwGetWindowAttrib(Global.engine.glfwWindow, GLFW_FOCUSED) == GLFW_TRUE) {
    //                 if (io.keyboard[IO.Key.A]!!.Down) {
    //                     keyInput[0] -= 1.0f;
    //                 }
    //                 if (io.keyboard[IO.Key.D]!!.Down) {
    //                     keyInput[0] += 1.0f;
    //                 }
    //                 if (io.keyboard[IO.Key.W]!!.Down) {
    //                     keyInput[1] += 1.0f;
    //                 }
    //                 if (io.keyboard[IO.Key.S]!!.Down) {
    //                     keyInput[1] -= 1.0f;
    //                 }
    //                 if (io.keyboard[IO.Key.Q]!!.Down) {
    //                     keyInputRoll -= 1.0f;
    //                 }
    //                 if (io.keyboard[IO.Key.E]!!.Down) {
    //                     keyInputRoll += 1.0f;
    //                 }
    //                 if (io.keyboard[IO.Key.LShift]!!.Down) {
    //                     localSpeed *= 2.0f;
    //                 }
    // //                if (keyInput[0] == 0 && keyInput[1] == 0 && keyInputRoll == 0) {
    // //                    return false;
    // //                }

    //                 val deltaDir = dir.copy() * keyInput[1] * localSpeed
    //                 val deltaRight = right.copy() * keyInput[0] * localSpeed

    //                 eyePos += deltaRight + deltaDir

    //                 roll += keyInputRoll * localSpeed * 0.1f

    //                 //float* up = Math::normalize(Math::cross(dir, right));
    //                 //up = Math::multiply(up, keyInput[0]);
    //             }
  }

  pub fn update_matrices(&mut self) {
    self.prev_view_mat= self.view_mat;
    self.prev_proj_mat= self.proj_mat;
    self.prev_view_proj_mat= self.view_proj_mat;
    self.prev_inv_view_mat= self.inv_view_mat;
    self.prev_inv_proj_mat= self.inv_proj_mat;

    let mut pos = self.eye_pos;
    let mut target = self.target_pos;

    pos.x *= -1.;
    target.x *= -1.;

    // self.view_mat = look_at(&self.pos, &self.look_at, &self.up);
    self.view_mat = look_at(&pos, &target, &self.up);

    self.proj_mat = nalgebra_glm::perspective_fov(
    // self.fov, 
  radians(&vec1(self.fov))[0],
    self.width as f32, self.height as f32, self.near, self.far);
    // self.proj_mat = nalgebra_glm::perspective(
    //   self.aspect_ratio,
    //   radians(&vec1(self.fov))[0],
    //   self.near,
    //   self.far,
    // );
    self.proj_mat[(1, 1)] *= -1.0;

    self.inv_view_mat = self.view_mat.clone().try_inverse().unwrap();  
    self.inv_proj_mat = self.proj_mat.clone().try_inverse().unwrap();  
    self.view_proj_mat = self.proj_mat * self.view_mat;
    

  }
}
