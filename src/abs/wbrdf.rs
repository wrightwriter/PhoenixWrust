use ash::vk::{self, Rect2D, Offset2D, Extent2D};

use crate::{sys::{warenaitems::{WAIdxImage, WArenaItem}, wtl::WTechLead}, wvulkan::WVulkan, res::img::{wimage::WImageInfo, wrendertarget::{WRTInfo, WRPConfig}}};

use super::{ passes::{wfxpass::WFxPass, wpostpass::WPassTrait}};




pub struct WBrdf{
  pub brdf: WAIdxImage,
}

impl WBrdf {
  pub fn new(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
  ) -> Self {
    let prog = w_v.w_shader_man.new_render_program(&mut w_v.w_device, "fullscreenQuad.vert", "brdf.frag");
    let mut pass = WFxPass::new(w_v, w_tl, false, prog);
    
    let rt = w_tl.new_render_target(w_v, WRTInfo { 
      resx: 512, resy: 512, 
      format: vk::Format::R16G16_SFLOAT, 
      ..wdef!()
    }).0;

    unsafe {
      let cmd_buf = pass.run_on_external_rt(rt, w_v, w_tl);
      
      w_v.w_device.single_command_submit(cmd_buf);
      w_v.w_device.device.queue_wait_idle(w_v.w_device.queue);
    }

    Self { brdf: rt.get().image_at(0) }
  }

}





