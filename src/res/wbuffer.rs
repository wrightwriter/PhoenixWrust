
use ash::vk;

use gltf::animation::util::rotations::F32;
use gpu_alloc::{GpuAllocator, MemoryBlock};
use gpu_alloc_ash::AshMemoryDevice;
use nalgebra_glm::{vec2, vec3, Vec3, Vec2, Mat4x4};

use crate::sys::warenaitems::WAIdxBuffer;

use super::wwritablebuffertrait::{WWritableBufferTrait, UniformValue, UniformsContainer};
use super::{wpongabletrait::WPongableTrait};


pub struct WBuffer {
  pub handles: [vk::Buffer;2],
  

  pub arena_index: WAIdxBuffer,

  memory_blocks: [MemoryBlock<vk::DeviceMemory>;2],

  pub descriptor_buffer_info: [vk::DescriptorBufferInfo;2],

  pub mapped_mems: [*mut u8;2],
  pub mapped_ptr_idx: *mut u8,

  pub sz_bytes: u32,

  pub bda_addresses: [vk::DeviceSize;2],

  pub mapped: bool,

  pub pongable: bool,
  pub pong_idx: u32,
}

impl WPongableTrait for WBuffer{
    fn pong(&mut self) {
      if self.pongable {
        self.pong_idx = 1 - self.pong_idx;
      }
    }

    fn is_pongable(&mut self)->bool {
      self.pongable
    }
}

impl WWritableBufferTrait for WBuffer{
    fn get_ptr(&mut self)->&mut *mut u8 {
      &mut self.mapped_ptr_idx
    }

    fn reset_ptr( &mut self) {
      self.mapped_ptr_idx = self.mapped_mems[self.pong_idx as usize];
    }
}

impl WBuffer {

  pub fn get_bda_address(
    &self,
  )-> vk::DeviceSize{
    self.bda_addresses[self.pong_idx as usize]
  }
  pub fn get_handle(
    &self,
  )-> vk::Buffer{
    self.handles[self.pong_idx as usize]
  }
  // TODO: prob borrow here?
  pub fn map(
    &mut self,
    device: &ash::Device,
  ){
    if self.mapped{
      return;
    }
    self.mapped = true;

    let backbuff_cnt = if self.pongable {2} else {1};

    for i in 0..backbuff_cnt{
      self.mapped_mems[i] = unsafe{
        self.memory_blocks[i].map(
        AshMemoryDevice::wrap(device),
          0, self.sz_bytes as usize
        ).expect("Coulnd't map buffer.")
      }.as_ptr();
    }
  }
  pub fn unmap(
    &mut self,
    device: &ash::Device,
  ){
    if !self.mapped{
      return;
    }
    self.mapped = false;
    let map_range = if self.pongable {2} else {1};
    for i in 0..map_range{
      unsafe{
        let mem = self.memory_blocks[i].memory();
        device.unmap_memory(*mem);
        self.mapped_mems[i] = std::ptr::null_mut() as *mut u8;
      }
      // self.mapped_mems[i] = unsafe{
      //   self.memory_blocks[i].map(
      //   AshMemoryDevice::wrap(device),
      //     0, self.sz_bytes as usize
      //   ).expect("Coulnd't map buffer.")
      // }.as_ptr();
    }
  }
  
  pub fn get_mapped_ptr(&mut self)->*mut u8{
    self.mapped_mems[self.pong_idx as usize]
  }
  // Not needed for now, because buff is coherent.
  pub fn flush(){
  }
  
  pub fn delete(
    &mut self,
    device: &ash::Device,
    allocator: &mut GpuAllocator<vk::DeviceMemory>,
  ){
    unsafe{
        let backbuff_cnt = if self.pongable {2} else {1};

        unsafe {
          for i in 0..backbuff_cnt{
            device.destroy_buffer(self.handles[i], None);
            // let mem_blk = & ;
            // ...
            // TODO: ???
            // mem leak lmao
            // allocator.dealloc( AshMemoryDevice::wrap(device), self.memory_blocks[i].clone());
          }
        }

    }
  }

  pub fn new(
    device: &ash::Device,
    allocator: &mut GpuAllocator<vk::DeviceMemory>,
    usage: vk::BufferUsageFlags,
    sz_bytes: u32,
    pongable: bool,
  ) -> Self {

    let flags = vk::ImageCreateFlags::empty();

    let vk_info = vk::BufferCreateInfo::builder()
      .size(sz_bytes as u64)
      .usage(
        usage | 
        vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS 
        )
      .sharing_mode(vk::SharingMode::EXCLUSIVE) // sharing between queues
      ;

    let mut flags = gpu_alloc::UsageFlags::HOST_ACCESS;
    flags.set(gpu_alloc::UsageFlags::DOWNLOAD, true);
    flags.set(gpu_alloc::UsageFlags::UPLOAD, true);
    flags.set(gpu_alloc::UsageFlags::DEVICE_ADDRESS, true);


    let backbuff_cnt = if pongable {2} else {1};
    
    let mut memory_blocks: [MemoryBlock<vk::DeviceMemory>;2] = wmemzeroed!();
    let mut bda_addresses : [vk::DeviceSize;2] = wmemzeroed!();
    let mut handles : [vk::Buffer;2] = wmemzeroed!();
    
    let mut descriptor_buffer_info: [vk::DescriptorBufferInfo;2] = wmemzeroed!();

    for i in 0..backbuff_cnt{
      let buffer = unsafe { device.create_buffer(&vk_info, None) }.unwrap();
      
      handles[i] = buffer;

      let mem_req = unsafe { device.get_buffer_memory_requirements(buffer) };

      let memory_block = unsafe {
        allocator.alloc(
            AshMemoryDevice::wrap(device),
            gpu_alloc::Request {
              size: mem_req.size,
              align_mask: mem_req.alignment - 1,
              // usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
              usage: flags,
              memory_types: mem_req.memory_type_bits,
            },
          )
          .unwrap()
      };
      unsafe {
        device.bind_buffer_memory(buffer, *memory_block.memory(), memory_block.offset());
      }
      memory_blocks[i] = memory_block;

      let bda_info = vk::BufferDeviceAddressInfo{
        buffer: buffer,
        ..Default::default()
      };
      let bda_address = unsafe{device.get_buffer_device_address(&bda_info)};
      descriptor_buffer_info[i] = vk::DescriptorBufferInfo::builder()
        .buffer(buffer)
        .offset(0)
        .range(sz_bytes.into())
        .build();
      


      bda_addresses[i] = bda_address;
    };

    if backbuff_cnt == 1 {
      bda_addresses[1] = bda_addresses[0];
      // memory_blocks[1] = memory_block;
      handles[1] = handles[0];
    }




    WBuffer {
      handles,
      memory_blocks,
      pongable,
      bda_addresses,
      descriptor_buffer_info,
      sz_bytes,
      mapped: false,
      pong_idx: 0,
      mapped_mems: wmemzeroed!(),
      mapped_ptr_idx: std::ptr::null_mut(),
      arena_index: wmemzeroed!(),
    }
  }
}

