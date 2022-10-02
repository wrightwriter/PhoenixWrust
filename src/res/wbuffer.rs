
use ash::vk;

use getset::Getters;
use gpu_alloc::{GpuAllocator, MemoryBlock};
use gpu_alloc_ash::AshMemoryDevice;


// !! ---------- IMAGE ---------- //

#[derive(Getters)]
pub struct WBuffer {
  pub handle: vk::Buffer,
  pub memory_block: MemoryBlock<vk::DeviceMemory>,
  pub sz_bytes: u32,
  pub bda_address: vk::DeviceSize,
  pub mapped: bool,
  pub mapped_array: Vec<f32>,
}

impl WBuffer {
  // TODO: prob borrow here?
  pub fn map(
    mut self,
    device: &ash::Device,
  )-> Self{
    self.mapped = true;

    let mapped_block = unsafe{
      self.memory_block.map(
      AshMemoryDevice::wrap(device),
        0, self.sz_bytes as usize
      ).expect("Coulnd't map buffer.")
    };

    unsafe {
      *(mapped_block.as_ptr() as *mut f32) = 1f32;
    }
    
    return self;
  }
  pub fn flush(){
  }
  pub fn new(
    device: &ash::Device,
    allocator: &mut GpuAllocator<vk::DeviceMemory>,
    usage: vk::BufferUsageFlags,
    sz_bytes: u32,
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


    let buffer = unsafe { device.create_buffer(&vk_info, None) }.unwrap();

    let mem_req = unsafe { device.get_buffer_memory_requirements(buffer) };


    let mut flags = gpu_alloc::UsageFlags::HOST_ACCESS;
    flags.set(gpu_alloc::UsageFlags::DOWNLOAD, true);
    flags.set(gpu_alloc::UsageFlags::UPLOAD, true);
    flags.set(gpu_alloc::UsageFlags::DEVICE_ADDRESS, true);



    let memory_block = unsafe {
      allocator.alloc(
          AshMemoryDevice::wrap(device),
          gpu_alloc::Request {
            size: mem_req.size,
            align_mask: mem_req.alignment - 1,
            // usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
            usage: flags,
            // Todo: make this safer? or not give a shit
            memory_types: mem_req.memory_type_bits,
          },
        )
        .unwrap()
    };

    unsafe {
      device.bind_buffer_memory(buffer, *memory_block.memory(), memory_block.offset());
    }

    let bda_info = vk::BufferDeviceAddressInfo{
      buffer: buffer,
      ..Default::default()
    };

    let bda_address = unsafe{device.get_buffer_device_address(&bda_info)};

    WBuffer {
      handle: buffer,
      memory_block,
      bda_address,
      sz_bytes,
      mapped: false,
      mapped_array: vec![],
    }
  }
}
// }

// impl Default for WImage{
//     fn default() -> Self {
//         Self { handle: None, resx: 500, resy: 500, format: None, view: None }
//     }
// }
