use nalgebra_glm::{Mat4x4, Vec2, Vec3, Vec4};

use crate::res::buff::wuniformscontainer::{WParamsContainer, WParamValue};



#[derive(Clone,Copy)]
pub enum UniformEnum {
  F32(f32),
  F64(f64),
  U64(u64),
  U32(u32),
  U16(u16),
  U8(u8),
  VEC2(Vec2),
  VEC3(Vec3),
  VEC4(Vec4),
  MAT4X4(Mat4x4),
  ARENAIDX(generational_arena::Index),
}

macro_rules! write {
  ($v: expr, $t: ty, $self: expr) => {
    unsafe {
      let ptr = $self.get_ptr();
      *(*ptr as *mut $t) = $v;
      *ptr = (*ptr as *mut $t).add(1) as *mut u8;
    }
  };
}
pub trait WWritableBufferTrait {
  fn get_ptr(&mut self) -> &mut *mut u8;
  fn reset_ptr(&mut self);

  fn write_params_container(
    &mut self,
    params_container: &WParamsContainer,
  ) {
    for uniform in &params_container.uniforms {
      match uniform {
        UniformEnum::F32(__) => self.write(*__),
        UniformEnum::F64(__) => self.write(*__),
        UniformEnum::U64(__) => self.write(*__),
        UniformEnum::U32(__) => self.write(*__),
        UniformEnum::U16(__) => self.write(*__),
        UniformEnum::U8(__) => self.write(*__),
        UniformEnum::VEC2(__) => self.write(*__),
        UniformEnum::VEC3(__) => self.write(*__),
        UniformEnum::VEC4(__) => self.write(*__),
        UniformEnum::MAT4X4(__) => self.write(*__),
        UniformEnum::ARENAIDX(__) => self.write(__.index as u8),
      };
    }
  }

  fn write<T: WParamValue>(
    &mut self,
    value: T,
  ) {
    match value.to_enum() {
      UniformEnum::F32(__) => write!(__, f32, self),
      UniformEnum::F64(__) => write!(__, f64, self),
      UniformEnum::U64(__) => write!(__, u64, self),
      UniformEnum::U32(__) => write!(__, u32, self),
      UniformEnum::U16(__) => write!(__, u16, self),
      UniformEnum::U8(__) => write!(__, u8, self),
      UniformEnum::VEC2(__) => self.write_vec2(__),
      UniformEnum::VEC3(__) => self.write_vec3(__),
      UniformEnum::VEC4(__) => self.write_vec4(__),
      UniformEnum::MAT4X4(__) => self.write_mat4x4(__),
      UniformEnum::ARENAIDX(__) => write!(__.index as u8 - 1, u8, self),
    };
  }

  // fn write_float(&mut self, value: f32 ) {
  //   self.write_value(value);
  // }

  // fn write_u64(&mut self, value: u64 ) {
  //   self.write_value(value);
  // }

  // fn write_u32(&mut self, value: u32 ) {
  //   self.write_value(value);
  // }

  // fn write_u8(&mut self, value: u8 ) {
  //   self.write_value(value);
  // }

  fn write_vec2(
    &mut self,
    value: Vec2,
  ) {
    unsafe {
      write!(value[0], f32, self);
      write!(value[1], f32, self);
    }
  }

  fn write_vec3(
    &mut self,
    value: Vec3,
  ) {
    unsafe {
      write!(value[0], f32, self);
      write!(value[1], f32, self);
      write!(value[2], f32, self);
    }
  }

  fn write_vec4(
    &mut self,
    value: Vec4,
  ) {
    unsafe {
      write!(value[0], f32, self);
      write!(value[1], f32, self);
      write!(value[2], f32, self);
      write!(value[3], f32, self);
    }
  }

  fn write_mat4x4(
    &mut self,
    value: Mat4x4,
  ) {
    unsafe {
      write!(value[0], f32, self);
      write!(value[1], f32, self);
      write!(value[2], f32, self);
      write!(value[3], f32, self);
      write!(value[4], f32, self);
      write!(value[5], f32, self);
      write!(value[6], f32, self);
      write!(value[7], f32, self);
      write!(value[8], f32, self);
      write!(value[9], f32, self);
      write!(value[10], f32, self);
      write!(value[11], f32, self);
      write!(value[12], f32, self);
      write!(value[13], f32, self);
      write!(value[14], f32, self);
      write!(value[15], f32, self);
    }
  }
}
