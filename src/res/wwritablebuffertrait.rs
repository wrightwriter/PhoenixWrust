use nalgebra_glm::{Vec2, Vec3, Mat4x4, Vec4};
use smallvec::SmallVec;


pub enum UniformEnum{
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
}



pub trait UniformValue{
  fn get_enum(&self)->UniformEnum;
}
impl UniformValue for f32{
    fn get_enum(&self) -> UniformEnum { UniformEnum::F32(*self) }
}
impl UniformValue for f64{
    fn get_enum(&self) -> UniformEnum { UniformEnum::F64(*self) }
}
impl UniformValue for u64{
    fn get_enum(&self) -> UniformEnum { UniformEnum::U64(*self) }
}
impl UniformValue for u32{
    fn get_enum(&self) -> UniformEnum { UniformEnum::U32(*self) }
}
impl UniformValue for u16{
    fn get_enum(&self) -> UniformEnum { UniformEnum::U16(*self) }
}
impl UniformValue for u8{
    fn get_enum(&self) -> UniformEnum { UniformEnum::U8(*self) }
}

impl UniformValue for Vec2{
    fn get_enum(&self) -> UniformEnum { UniformEnum::VEC2(*self) }
}
impl UniformValue for Vec3{
    fn get_enum(&self) -> UniformEnum { UniformEnum::VEC3(*self) }
}
impl UniformValue for Vec4{
    fn get_enum(&self) -> UniformEnum { UniformEnum::VEC4(*self) }
}

impl UniformValue for Mat4x4{
    fn get_enum(&self) -> UniformEnum { UniformEnum::MAT4X4(*self) }
}


pub struct UniformsContainer{
  pub uniforms: SmallVec<[UniformEnum;32]>
}
impl UniformsContainer{
  pub fn new()->Self{
    let uniforms = SmallVec::new();
    Self { uniforms }
  }
  pub fn add<T: UniformValue>(&mut self, t: T){
    self.uniforms.push(t.get_enum());
  }
  pub fn set_at<T: UniformValue>(&mut self, idx: usize, t: T){
    self.uniforms[idx] = t.get_enum();
  }
  pub fn reset(&mut self){
    unsafe{
      self.uniforms.set_len(0);
    }
  }
}

macro_rules! write {
  ($v: expr, $t: ty, $self: expr) => {
    unsafe{
      let ptr = $self.get_ptr();
      *(*ptr as *mut $t) = $v;
      *ptr = (*ptr as *mut $t).add(1) as *mut u8;
    }
  };
}
pub trait WWritableBufferTrait{
  
  fn get_ptr(&mut self)->&mut *mut u8;
  fn reset_ptr(&mut self);
  
  
  fn write_uniforms_container(&mut self, uniforms_container: &UniformsContainer){
    for uniform in &uniforms_container.uniforms{
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
      };
    }
  }

  
  fn write<T: UniformValue>(&mut self, value: T){
    match value.get_enum() {
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

  fn write_vec2(&mut self, value: Vec2 ) {
      unsafe{
        write!(value[0], f32, self); 
        write!(value[1], f32, self); 
      }
  }

  fn write_vec3(&mut self, value: Vec3) {
      unsafe{
        write!(value[0], f32, self); 
        write!(value[1], f32, self); 
        write!(value[2], f32, self); 
      }
  }

  fn write_vec4(&mut self, value: Vec4) {
      unsafe{
        write!(value[0], f32, self); 
        write!(value[1], f32, self); 
        write!(value[2], f32, self); 
        write!(value[3], f32, self); 
      }
  }


  fn write_mat4x4(&mut self, value: Mat4x4 ) {
      unsafe{
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