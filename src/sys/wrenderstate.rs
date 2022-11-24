use ash::vk;

use super::wdevice::WDevice;

pub struct WRenderState {
  pub depth_test: bool,
  pub depth_write: bool,
  pub depth_compare: vk::CompareOp,
  pub cull_mode: vk::CullModeFlags,
  pub blend_constants: Option<[f32;4]>,
}
impl WRenderState {
  pub fn run(
    &self,
    cmd_buf: vk::CommandBuffer,
    w_device: &mut WDevice,
  ) {
    unsafe {
      w_device.device.cmd_set_depth_test_enable(cmd_buf, self.depth_test);
      w_device.device.cmd_set_depth_write_enable(cmd_buf, self.depth_write);
      w_device.device.cmd_set_cull_mode(cmd_buf, self.cull_mode);
      w_device.device.cmd_set_depth_compare_op(cmd_buf, self.depth_compare);
      if let Some(blend_constants) = self.blend_constants{
        w_device.device.cmd_set_blend_constants(cmd_buf, &blend_constants);
      }
    }
  }
}
impl Default for WRenderState {
  fn default() -> Self {
    Self {
      depth_test: true,
      depth_write: true,
      cull_mode: vk::CullModeFlags::BACK,
      blend_constants: None,
      depth_compare: vk::CompareOp::LESS_OR_EQUAL,
    }
  }
}
