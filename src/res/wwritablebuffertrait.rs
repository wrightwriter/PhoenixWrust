use nalgebra_glm::{Vec2, Vec3, Mat4x4};


pub trait WWritableBufferTrait{
  
  fn get_ptr(&mut self)->&mut *mut u8;
  fn reset_ptr(&mut self);

  fn write_float(&mut self, value: f32 ) {
      unsafe{
        let ptr = self.get_ptr();
        *(*ptr as *mut f32) = value;
        *ptr = (*ptr as *mut f32).add(1) as *mut u8;
        // $mem_ptr = $mem_ptr.add(1);
      }
  }

  fn write_u64(&mut self, value: u64 ) {
      unsafe{
        let ptr = self.get_ptr();
        *(*ptr as *mut u64) = value;
        *ptr = (*ptr as *mut u64).add(1) as *mut u8;
        // $mem_ptr = $mem_ptr.add(1);
      }
  }

  fn write_uint(&mut self, value: u32 ) {
      unsafe{
        let mut ptr = self.get_ptr();
        *(*ptr as *mut u32) = value;
        *ptr = (*ptr as *mut u32).add(1) as *mut u8;
      }
  }

  fn write_vec2(&mut self, value: Vec2 ) {
      unsafe{
        self.write_float(value[0]); 
        self.write_float(value[1]); 
      }
  }

  fn write_vec3(&mut self, value: Vec3) {
      unsafe{
        self.write_float(value[0]); 
        self.write_float(value[1]); 
        self.write_float(value[2]); 
      }
  }

  fn write_mat4x4(&mut self, value: Mat4x4 ) {
      unsafe{
        self.write_float(value[0]); 
        self.write_float(value[1]); 
        self.write_float(value[2]); 
        self.write_float(value[3]); 
        self.write_float(value[4]); 
        self.write_float(value[5]); 
        self.write_float(value[6]); 
        self.write_float(value[7]); 
        self.write_float(value[8]); 
        self.write_float(value[9]); 
        self.write_float(value[10]); 
        self.write_float(value[11]); 
        self.write_float(value[12]); 
        self.write_float(value[13]); 
        self.write_float(value[14]); 
        self.write_float(value[15]); 
      }
  }
}