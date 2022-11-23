
use ash::vk;

use gpu_alloc::GpuAllocator;
use crate::res::buff::wbuffer::WBuffer;


// use crate::wbuffer::WBuffer;
use crate::sys::wdevice::WDevice;
use crate::res::img::wimage::WImage;
use crate::sys::warenaitems::{WAIdxImage, WAIdxBuffer};

use super::buff::wuniformscontainer::WParamsContainer;
use super::wpongabletrait::WPongableTrait;

pub trait WBindingAttachmentTrait {
  fn get_binding_type(&self) -> vk::DescriptorType;
}

pub struct WBindingUBO {
  pub buff: WBuffer,
  pub uniforms: WParamsContainer,
  pongable: bool,
}
impl WBindingUBO {
  pub fn new(
    device: &ash::Device,
    allocator: &mut GpuAllocator<vk::DeviceMemory>,
    sz_bytes: u32,
  ) -> Self {
    let mut buff = WBuffer::new(
      device,
      allocator,
      vk::BufferUsageFlags::UNIFORM_BUFFER,
      sz_bytes,
      true
    );
    buff.map(device);


    Self { buff, pongable: true, uniforms: WParamsContainer::new() }
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
  pub idx_counter: u32,
  pub vk_infos: Vec<vk::DescriptorBufferInfo>,
  pub dummy_buff: WAIdxBuffer,
}
impl WBindingBufferArray {
  pub fn new(
    w_device: &mut WDevice,
    dummy_buff: (&WBuffer, &WAIdxBuffer ),
    max_size: u32,
  ) -> Self {
    let mut vk_infos = Vec::with_capacity(max_size as usize);

    for i in 0..max_size {
      vk_infos.push(
        vk::DescriptorBufferInfo::builder()
          .buffer(dummy_buff.0.get_handle())
          .range(dummy_buff.0.sz_bytes.into())
          .offset(0)
          .build(),
      )
    }

    Self {
      count: max_size,
      idx_counter: 0,
      vk_infos,
      dummy_buff: *dummy_buff.1,
    }
  }
}

pub struct WBindingImageArray {
  pub count: u32,
  pub idx_counter: u32,
  pub vk_infos_storage: Vec<vk::DescriptorImageInfo>,
  pub vk_infos_sampled: Vec<vk::DescriptorImageInfo>,
  pub dummy_image: WAIdxImage,
}
impl WBindingImageArray {
  pub fn new(
    w_device: &mut WDevice,
    dummy_image: (&WImage, &WAIdxImage ),
    max_size: u32,
  ) -> Self {
    // let mut vk_infos = Vec::with_capacity(count as usize);
    let mut vk_infos_storage = vec![];
    let mut vk_infos_sampled = vec![];
    

    for i in 0..max_size {
      vk_infos_storage.push(
          dummy_image.0.descriptor_image_info
      );
      vk_infos_sampled.push(
          dummy_image.0.descriptor_image_info
      );
    }

    Self {
      count: max_size,
      idx_counter: 1,
      vk_infos_storage,
      vk_infos_sampled,
      dummy_image: *dummy_image.1,
    }
  }
}