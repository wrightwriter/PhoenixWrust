use smallvec::SmallVec;
use nalgebra_glm::{Mat4x4, Vec2, Vec3, Vec4};
use crate::res::buff::wwritablebuffertrait::UniformEnum;
use crate::sys::warenaitems::{WAIdxBuffer, WAIdxImage, WAIdxRt, WAIdxUbo, WArenaItem};

use super::wwritablebuffertrait::WWritableBufferTrait;

pub trait WUniformValue {
  fn to_enum(&self) -> UniformEnum;
}

impl WUniformValue for f32 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::F32(*self)
  }
}

impl WUniformValue for f64 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::F64(*self)
  }
}

impl WUniformValue for u64 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::U64(*self)
  }
}

impl WUniformValue for u32 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::U32(*self)
  }
}

impl WUniformValue for u16 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::U16(*self)
  }
}

impl WUniformValue for u8 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::U8(*self)
  }
}

impl WUniformValue for Vec2 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::VEC2(*self)
  }
}

impl WUniformValue for Vec3 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::VEC3(*self)
  }
}

impl WUniformValue for Vec4 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::VEC4(*self)
  }
}

impl WUniformValue for Mat4x4 {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::MAT4X4(*self)
  }
}

impl WUniformValue for WAIdxUbo {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

impl WUniformValue for WAIdxImage {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

impl WUniformValue for WAIdxBuffer {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

impl WUniformValue for WAIdxRt {
  fn to_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

#[derive(Clone)]
pub struct WUniformsContainer {
  pub uniforms: SmallVec<[UniformEnum; 32]>,
  pub uniforms_names: SmallVec<[smallstr::SmallString<[u8;30]>; 32]>,
  pub exposed: bool,
}

impl WUniformsContainer {
  pub fn new() -> Self {
    let uniforms = SmallVec::new();
    let uniforms_names = SmallVec::new();
    let a:smallstr::SmallString<[u8;30]> = "  ".into();

    Self { uniforms, uniforms_names, exposed: false }
  }
  pub fn update_uniforms(ubo: WAIdxUbo, uniforms: &WUniformsContainer){
    // -- UBO -- //
    let ubo = &mut ubo.get_mut().buff;
    ubo.reset_ptr();
    ubo.write_uniforms_container(&uniforms);
  }
  pub fn add<T: WUniformValue>(
    &mut self,
    t: T,
  ) {
    self.uniforms.push(t.to_enum());
  }
  pub fn add_many<T: WUniformValue>(
    &mut self,
    t_arr: &[T],
  ) {
    for t in t_arr{
      self.uniforms.push(t.to_enum());
    }
  }
  pub fn set_at<T: WUniformValue>(
    &mut self,
    idx: usize,
    t: T,
  ) {
    self.uniforms[idx] = t.to_enum();
  }
  pub fn reset(&mut self) {
    unsafe {
      self.uniforms.set_len(0);
    }
  }
}
