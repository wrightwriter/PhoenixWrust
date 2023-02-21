use std::ops::Index;

use smallvec::SmallVec;
use nalgebra_glm::{Mat4x4, Vec2, Vec3, Vec4};
use crate::res::buff::wwritablebuffertrait::UniformEnum;
use crate::sys::warenaitems::{WAIdxBuffer, WAIdxImage, WAIdxRt, WAIdxUbo, WArenaItem};

use super::wwritablebuffertrait::WWritableBufferTrait;

pub trait WParamValue {
  fn to_enum(&self) -> UniformEnum;
}

impl WParamValue for UniformEnum {
  fn to_enum(&self) -> UniformEnum {
    *self 
  }
}

impl WParamValue for f32 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::F32(*self)
  }
}

impl WParamValue for f64 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::F64(*self)
  }
}


impl WParamValue for u64 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::U64(*self)
  }
}

impl WParamValue for u32 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::U32(*self)
  }
}

impl WParamValue for u16 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::U16(*self)
  }
}

impl WParamValue for u8 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::U8(*self)
  }
}

impl WParamValue for Vec2 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::VEC2(*self)
  }
}

impl WParamValue for Vec3 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::VEC3(*self)
  }
}

impl WParamValue for Vec4 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::VEC4(*self)
  }
}

impl WParamValue for Mat4x4 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::MAT4X4(*self)
  }
}

impl WParamValue for WAIdxUbo {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

impl WParamValue for WAIdxImage {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

impl WParamValue for WAIdxBuffer {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

impl WParamValue for WAIdxRt {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

#[derive(Clone)]
pub struct WParamsContainer {
  pub uniforms: SmallVec<[UniformEnum; 32]>,
  pub uniforms_names: SmallVec<[smallstr::SmallString<[u8;30]>; 32]>,
  pub exposed: bool,
}

impl WParamsContainer {
  pub fn new() -> Self {
    let uniforms = SmallVec::new();
    let uniforms_names = SmallVec::new();
    let a:smallstr::SmallString<[u8;30]> = "  ".into();

    Self { uniforms, uniforms_names, exposed: false }
  }
  pub fn reset_ptr(ubo: WAIdxUbo){
    let ubo = &mut ubo.get_mut().buff;
    ubo.reset_ptr();
  }
  pub fn upload_uniforms(ubo: WAIdxUbo, uniforms: &WParamsContainer){
    // -- UBO -- //
    let ubo = &mut ubo.get_mut().buff;
    ubo.reset_ptr();
    ubo.write_params_container(&uniforms);
  }
  pub fn add<T: WParamValue>(
    &mut self,
    t: T,
  ) {
    self.uniforms.push(t.to_enum());
  }
  pub fn add_many<T: WParamValue>(
    &mut self,
    t_arr: &[T],
  ) {
    for t in t_arr{
      self.uniforms.push(t.to_enum());
    }
  }
  pub fn set_at<T: WParamValue>(
    &mut self,
    idx: usize,
    t: T,
  ) {
    self.uniforms[idx] = t.to_enum();
  }
  pub fn set_len(&mut self, len: usize) {
    unsafe {
      self.uniforms.set_len(len);
    }
  }
  pub fn len(&mut self) -> usize {
    self.uniforms.len()
  }
  pub fn reset(&mut self) {
    unsafe {
      self.uniforms.set_len(0);
    }
  }
}

impl Index<usize> for WParamsContainer {
    type Output = UniformEnum;
    fn index(&self, i: usize) -> &UniformEnum {
      &self.uniforms[i]
    }
}
