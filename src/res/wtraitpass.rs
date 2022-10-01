use std::{
  borrow::BorrowMut,
  cell::Cell,
  ffi::CStr,
  mem::MaybeUninit,
  ops::{Deref, DerefMut},
};

use ash::{
  vk,
  vk::{GraphicsPipelineCreateInfoBuilder, Rect2DBuilder},
  // ExtendableFrom,
};

use getset::Getters;
use gpu_alloc::GpuAllocator;
use gpu_alloc_ash::AshMemoryDevice;

use crate::{
  wmemzeroed,
  res::wshader::{WProgram, WShader},
};

static entry_point: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

pub struct WTraitPass {}

impl WTraitPass {
  fn new(device: &ash::Device) -> Self {
    Self {}
  }
}

impl WTraitPass {}
