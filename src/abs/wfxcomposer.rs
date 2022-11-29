

use ash::vk;
use smallvec::SmallVec;

use crate::{
  res::{
    img::{
      wimage::WImageInfo,
      wrendertarget::{WRTInfo},
    },
    wpongabletrait::WPongableTrait,
  },
  sys::{
    warenaitems::{
      WAIdxImage, WAIdxRt, WArenaItem,
    },
    wbarr::WBarr, wtl::WTechLead,
  },
  wvulkan::WVulkan,
};

use super::passes::wpostpass::WPassTrait;

// use super::wpostpass::WPassTrait;
// use crate::;

pub struct WFxComposer {
  pub rt: WAIdxRt,
  rt_in: WAIdxRt,
  pub cmd_bufs: SmallVec<[vk::CommandBuffer; 31]>,
}

impl WFxComposer {
  pub fn new(w: &mut WVulkan, w_tl: &mut WTechLead ) -> Self {
    let rt_info = WRTInfo {
      resx: w.w_cam.width,
      resy: w.w_cam.height,
      pongable: true,
      has_depth: false,
      attachment_infos: vec![WImageInfo { ..wdef!() }],
      ..wdef!()
    };
    let rt = w_tl.new_render_target(w, rt_info);

    Self {
      rt: rt.0,
      cmd_bufs: SmallVec::new(),
      rt_in: wmemzeroed!(),
    }
  }
  pub fn get_front_img(&self) -> WAIdxImage {
    self.rt.get().image_at(0)
  }
  pub fn get_back_img(&self) -> WAIdxImage {
    self.rt.get().back_image_at(0)
  }

  pub fn begin(
    &mut self,
    rt_in: WAIdxRt,
  ) {
    unsafe {
      self.cmd_bufs.set_len(0);
      self.rt_in = rt_in;
    }
  }

  pub fn end(&self) {}

  pub fn run(
    &mut self,
    w_v: &mut WVulkan,
    w_t_l: &mut WTechLead,
    fx_pass: &mut impl WPassTrait,
  ) {
    let rt = self.rt.get_mut();
    rt.pong();

    // TODO: CODE DUPLICATION
    if fx_pass.get_rt().is_none() {
      fx_pass.set_rt(self.rt);
      let rp =fx_pass.get_pipeline().get_mut();
      rp.set_pipeline_render_target(rt);
      rp.refresh_pipeline(&w_v.w_device.device, &w_t_l);
    }

    let pc = fx_pass.get_push_constants();
    pc.reset();

    let mut img_idx;
    if self.cmd_bufs.len() == 0 {
      img_idx = self.rt_in.get().image_at(0);
    } else {
      img_idx = rt.image_indices[1-rt.pong_idx as usize][0];
      // img_idx.idx.index -= 0;
      // println!("{}",img_idx.idx.index);
    }
    pc.add_many(&[img_idx, img_idx, img_idx, img_idx]);

    let cmd_buf = rt.begin_pass(&mut w_v.w_device);
    
    
    // if let Some(rt) = rt {
    
    // }
    fx_pass.run(w_v,w_t_l, &cmd_buf);
    rt.end_pass(&mut w_v.w_device);
    self.cmd_bufs.push(rt.cmd_buf);


    let bar_cmd_buf = w_v.w_device.curr_pool().get_cmd_buff();

    let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();
    unsafe {
      w_v
        .w_device
        .device
        .begin_command_buffer(bar_cmd_buf, &cmd_buf_begin_info);
    }
    let bar = WBarr::render().run_on_cmd_buff(&w_v.w_device, bar_cmd_buf);

    unsafe {
      w_v.w_device.device.end_command_buffer(bar_cmd_buf);
    }

    self.cmd_bufs.push(bar_cmd_buf);
  }
}
