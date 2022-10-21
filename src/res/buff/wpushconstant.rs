use super::wwritablebuffertrait::WWritableBufferTrait;



pub struct WPushConstant{
    pub array: [u8; 256],
    array_ptr: *mut u8,
    // pub start_ptr: *mut u8,
}

impl WPushConstant{
    pub fn new()->Self{
        let mut array = [0;256];
        Self{
            array,
            array_ptr: std::ptr::null_mut(),
        }
    }    
    // very unsafe
    pub fn init(&mut self){
        self.array_ptr = self.array.as_mut_ptr();
    }
}

impl WWritableBufferTrait for WPushConstant{
    fn get_ptr(&mut self) -> &mut *mut u8 {
      &mut self.array_ptr
    }

    fn reset_ptr( &mut self) {
      self.array_ptr = self.array.as_mut_ptr();
    }
}