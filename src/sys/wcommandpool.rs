use ash::{
  vk::{
    self, CommandPoolResetFlags,
  },
};



pub struct WCommandPool{
  pub command_pool: vk::CommandPool,
  pub command_buffers: Vec<vk::CommandBuffer>,
  command_buffs_idx: usize,
}

impl WCommandPool{
  pub fn get_cmd_buff(
    &mut self,
    )-> vk::CommandBuffer{
    // lmaooooooooooooooooooooooo
    self.command_buffs_idx += 1;
    self.command_buffers[self.command_buffs_idx - 1]
  }
  pub fn new(
    device: &ash::Device,
    queue_family: u32,
  )->Self{
    let command_pool_info = vk::CommandPoolCreateInfo::builder()
      .queue_family_index(queue_family)
      .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
    
    let command_pool = unsafe { device.create_command_pool(&command_pool_info, None) }.unwrap();

    let command_buffers = unsafe {
      let cmd_buf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1000);
      device
        .allocate_command_buffers(&cmd_buf_allocate_info).unwrap()
    };

    Self { command_pool, command_buffers,command_buffs_idx: 0}
  }
  
  pub fn reset(
    &mut self,
    device: &ash::Device,
  ){
    unsafe{
      device.reset_command_pool(self.command_pool, CommandPoolResetFlags::empty());
    }
    self.command_buffs_idx = 0;
  }
}