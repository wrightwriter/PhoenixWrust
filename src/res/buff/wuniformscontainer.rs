use smallvec::SmallVec;
use nalgebra_glm::{Mat4x4, Vec2, Vec3, Vec4};
use crate::res::buff::wwritablebuffertrait::UniformEnum;
use crate::sys::warenaitems::{WAIdxBuffer, WAIdxImage, WAIdxRt, WAIdxUbo};

pub trait UniformValue {
  fn get_enum(&self) -> UniformEnum;
}

impl UniformValue for f32 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::F32(*self)
  }
}

impl UniformValue for f64 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::F64(*self)
  }
}

impl UniformValue for u64 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::U64(*self)
  }
}

impl UniformValue for u32 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::U32(*self)
  }
}

impl UniformValue for u16 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::U16(*self)
  }
}

impl UniformValue for u8 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::U8(*self)
  }
}

impl UniformValue for Vec2 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::VEC2(*self)
  }
}

impl UniformValue for Vec3 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::VEC3(*self)
  }
}

impl UniformValue for Vec4 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::VEC4(*self)
  }
}

impl UniformValue for Mat4x4 {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::MAT4X4(*self)
  }
}

impl UniformValue for WAIdxUbo {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

impl UniformValue for WAIdxImage {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

impl UniformValue for WAIdxBuffer {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

impl UniformValue for WAIdxRt {
  fn get_enum(&self) -> UniformEnum {
    UniformEnum::ARENAIDX(self.idx)
  }
}

pub struct UniformsContainer {
  pub uniforms: SmallVec<[UniformEnum; 32]>,
}

impl UniformsContainer {
  pub fn new() -> Self {
    let uniforms = SmallVec::new();
    Self { uniforms }
  }
  pub fn add<T: UniformValue>(
    &mut self,
    t: T,
  ) {
    self.uniforms.push(t.get_enum());
  }
  pub fn set_at<T: UniformValue>(
    &mut self,
    idx: usize,
    t: T,
  ) {
    self.uniforms[idx] = t.get_enum();
  }
  pub fn reset(&mut self) {
    unsafe {
      self.uniforms.set_len(0);
    }
  }
}
