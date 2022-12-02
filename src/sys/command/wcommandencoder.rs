use ash::vk::{self};

use crate::{sys::wdevice::WDevice, wvulkan::WVulkan};

use smallvec::SmallVec;

use super::{wbarr::WBarr, wsemaphore::WSemaphore};

pub struct WCommandEncoder {
  // pub command_buffs: SmallVec<[vk::CommandBuffer;40]>,
  pub cmd_bufs: SmallVec<[vk::CommandBufferSubmitInfo; 32]>,
}

impl WCommandEncoder {
  pub fn new() -> Self {
    Self {
      cmd_bufs: SmallVec::new(),
    }
  }

  pub fn get_and_begin_buff(
    &mut self,
    w_device: &mut WDevice,
  ) -> vk::CommandBuffer {
    let cmd_buff = w_device.curr_pool().get_cmd_buff();
    unsafe {
      let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();
      w_device
        .device
        .begin_command_buffer(cmd_buff, &cmd_buf_begin_info);
    }
    cmd_buff
  }

  pub fn end_and_push_buff(
    &mut self,
    w_device: &mut WDevice,
    command_buff: vk::CommandBuffer,
  ) {
    unsafe {
      w_device.device.end_command_buffer(command_buff);
    }
    self.cmd_bufs.push(
      vk::CommandBufferSubmitInfo::builder()
        .command_buffer(command_buff)
        .build(),
    );
  }

  pub fn push_bufs(
    &mut self,
    cmd_bufs: &[vk::CommandBuffer],
  ) {
    for buf in cmd_bufs {
      self.cmd_bufs.push(
        vk::CommandBufferSubmitInfo::builder()
          .command_buffer(*buf)
          .build(),
      );
    }
  }

  pub fn push_buf(
    &mut self,
    cmd_buf: vk::CommandBuffer,
  ) {
    self.cmd_bufs.push(
      vk::CommandBufferSubmitInfo::builder()
        .command_buffer(cmd_buf)
        .build(),
    );
  }

  pub fn push_barr(
    &mut self,
    // w_device: &mut WDevice,
    w_v: &mut WVulkan,
    mut barrier: WBarr,
  ) {
    self.push_buf(barrier.run_on_new_cmd_buff(w_v))
  }

  pub fn submit_to_queue(
    &mut self,
    w_device: &WDevice,
  ) {
    let submit_info = vk::SubmitInfo2::builder()
      .command_buffer_infos(&self.cmd_bufs)
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
      .command_buffer_infos(&self.cmd_bufs)
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

  pub fn reset(
    &mut self,
    w_device: &WDevice,
  ) {
    unsafe {
      // self.command_buffs.iter().map(|__| {
      //   w_device.device.free_command_buffers(w_device.command_pool, &[__.command_buffer]);
      // });
      self.cmd_bufs.set_len(0);
    }
  }
  // pub fn add_semaphore(&mut self, semaphore: &mut WSemaphore){
  //   self.command_buffs.push(command_buff);
  // }
}
