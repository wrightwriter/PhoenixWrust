use std::{
  ffi::CStr,
};




static entry_point: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

pub struct WTraitPass {}

impl WTraitPass {
  fn new(device: &ash::Device) -> Self {
    Self {}
  }
}

impl WTraitPass {}
