use ash::{
  vk::{
    self,
  },
};


use crate::{
  sys::{
    wdevice::WDevice,
  },
};

use smallvec::SmallVec;


use super::{wbarr::WBarr, wsemaphore::WSemaphore};

pub struct WCommandEncoder {
  // pub command_buffs: SmallVec<[vk::CommandBuffer;40]>,
  pub command_buffs: SmallVec<[vk::CommandBufferSubmitInfo; 32]>,
}

impl WCommandEncoder {
  pub fn new() -> Self {
    Self {
      command_buffs: SmallVec::new(),
    }
  }
  pub fn add_command(
    &mut self,
    command_buff: vk::CommandBuffer,
  ) {
    self.command_buffs.push(
      vk::CommandBufferSubmitInfo::builder()
        .command_buffer(command_buff)
        .build(),
    );
  }

  pub fn add_barr(
    &mut self,
    w_device: &WDevice,
    barrier: &WBarr,
    cmd_buff: vk::CommandBuffer
  ) {

    // TODO: not do this lmao
    unsafe {
      let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();
      w_device
        .device
        .begin_command_buffer(cmd_buff, &cmd_buf_begin_info);

      barrier.run(w_device, cmd_buff);

      w_device.device.end_command_buffer(cmd_buff);
    }
  }

  pub fn run(
    &mut self,
    w_device: &WDevice,
  ) {
    let submit_info = vk::SubmitInfo2::builder()
      .command_buffer_infos(&self.command_buffs)
      .build();

    unsafe {
      w_device
        .device
        .queue_submit2(w_device.queue, &[submit_info], vk::Fence::null())
        .unwrap();
    }
  }

  pub fn run_wait_semaphore(
    &mut self,
    w_device: &WDevice,
    semaphore: &mut WSemaphore,
    wait_value: u64,
  ) {
    let submit_info = vk::SubmitInfo2::builder()
      .command_buffer_infos(&self.command_buffs)
      .build();

    let wait_info = vk::SemaphoreWaitInfo::builder()
      .semaphores(&[semaphore.handle])
      .values(&[wait_value])
      .build();

    unsafe {
      w_device
        .device
        .queue_submit2(w_device.queue, &[submit_info], vk::Fence::null())
        .unwrap();
    }
  }
  
  pub fn reset( &mut self, w_device: &WDevice,){
    unsafe{
      // self.command_buffs.iter().map(|__| {
      //   w_device.device.free_command_buffers(w_device.command_pool, &[__.command_buffer]);
      // });
      self.command_buffs.set_len(0);
    }
  }
  // pub fn add_semaphore(&mut self, semaphore: &mut WSemaphore){
  //   self.command_buffs.push(command_buff);
  // }
}
