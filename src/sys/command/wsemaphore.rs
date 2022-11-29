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



#[derive(Clone, Copy)]
pub struct WSemaphore {
  pub handle: vk::Semaphore,
}
impl WSemaphore {
  pub fn new(w_device: &mut WDevice) -> Self {
    let mut type_info = vk::SemaphoreTypeCreateInfo::builder()
      .semaphore_type(vk::SemaphoreType::TIMELINE)
      .initial_value(0)
      .build();

    let info = vk::SemaphoreCreateInfo::builder().push_next(&mut type_info);

    let handle = unsafe { w_device.device.create_semaphore(&info, None).unwrap() };

    Self { handle }
  }

  pub fn signal_from_host(
    &self,
    w_device: &WDevice,
    signal_value: u64,
  ) {
    let signal_info = vk::SemaphoreSignalInfo::builder()
      .value(signal_value)
      .semaphore(self.handle)
      .build();
    unsafe {
      w_device.device.signal_semaphore(&signal_info);
    }
  }

  pub fn wait_from_host(
    &self,
    w_device: &WDevice,
    wait_value: u64,
  ) {
    let wait_info = vk::SemaphoreWaitInfo::builder()
      .semaphores(&[self.handle])
      .values(&[wait_value])
      .build();
    unsafe {
      w_device.device.wait_semaphores(&wait_info, u64::MAX);
    }
  }
  pub fn submit(
    &self,
    w_device: &mut WDevice,
  ) {
    let submit_info = vk::SubmitInfo2::builder();
    // .waitSe
  }
}
