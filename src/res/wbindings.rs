
use ash::vk;

use gpu_alloc::GpuAllocator;

use crate::res::wbuffer::WBuffer;
// use crate::wbuffer::WBuffer;
use crate::sys::wdevice::WDevice;
use crate::res::wimage::WImage;
use crate::sys::warenaitems::{WAIdxImage, WAIdxBuffer};

use super::wpongabletrait::WPongableTrait;

pub trait WBindingAttachmentTrait {
  fn get_binding_type(&self) -> vk::DescriptorType;
}

pub struct WBindingUBO {
  pub buff: WBuffer,
  pongable: bool,
}
impl WBindingUBO {
  pub fn new(
    device: &ash::Device,
    allocator: &mut GpuAllocator<vk::DeviceMemory>,
    sz_bytes: u32,
  ) -> Self {
    let buff = WBuffer::new(
      device,
      allocator,
      vk::BufferUsageFlags::UNIFORM_BUFFER,
      sz_bytes,
      true
    ).map(device);


    Self { buff, pongable: true }
  }
}

impl WPongableTrait for WBindingUBO{
    fn pong(&mut self) {
      self.buff.pong();
    }

    fn is_pongable(&mut self)->bool {
      self.pongable
    }
}

pub struct WBindingBufferArray {
  pub count: u32,
  pub idx: u32,
  pub vk_infos: Vec<vk::DescriptorBufferInfo>,
  pub dummy_buff: WAIdxBuffer,
}
impl WBindingBufferArray {
  pub fn new(
    w_device: &mut WDevice,
    dummy_buff: (&WBuffer, &WAIdxBuffer ),
    count: u32,
  ) -> Self {
    let mut vk_infos = Vec::with_capacity(count as usize);

    for i in 0..count {
      vk_infos.push(
        vk::DescriptorBufferInfo::builder()
          .buffer(dummy_buff.0.get_handle())
          .range(dummy_buff.0.sz_bytes.into())
          .offset(0)
          .build(),
      )
    }

    Self {
      count,
      idx: 0,
      vk_infos,
      dummy_buff: *dummy_buff.1,
    }
  }
}

pub struct WBindingImageArray {
  pub count: u32,
  pub idx_counter: u32,
  pub vk_infos: Vec<vk::DescriptorImageInfo>,
  pub dummy_image: WAIdxImage,
}
impl WBindingImageArray {
  pub fn new(
    w_device: &mut WDevice,
    dummy_image: (&WImage, &WAIdxImage ),
    count: u32,
  ) -> Self {
    // let mut vk_infos = Vec::with_capacity(count as usize);
    let mut vk_infos = vec![];
    

    for i in 0..count {
      vk_infos.push(
          dummy_image.0.descriptor_image_info
      )
    }

    Self {
      count,
      idx_counter: 0,
      vk_infos,
      dummy_image: *dummy_image.1,
    }
  }
}

// impl WBindingAttachmentTrait for WBindingImageArray {
//   fn get_binding_type(&self) -> vk::DescriptorType {
//     vk::DescriptorType::STORAGE_IMAGE
//   }
// }

// impl Default for WImage{
//     fn default() -> Self {
//         Self { handle: None, resx: 500, resy: 500, format: None, view: None }
//     }
// }
