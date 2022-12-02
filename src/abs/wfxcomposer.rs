

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
    command::wbarr::WBarr, wtl::WTechLead,
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
    w_tl: &mut WTechLead,
    fx_pass: &mut impl WPassTrait,
  ) {
    let mut img_idx;
    {
      let rt = self.rt.get_mut();
      rt.pong();
      if self.cmd_bufs.len() == 0 {
        img_idx = self.rt_in.get().image_at(0);
      } else {
        img_idx = rt.image_indices[1-rt.pong_idx as usize][0];
      }
    }

    let pc = fx_pass.get_push_constants();
    pc.reset();
    pc.add(img_idx);

    let cmd_bufs = fx_pass.run(w_v,w_tl, Some(img_idx), self.rt);
    for cmd_buf in &cmd_bufs{
      self.cmd_bufs.push(*cmd_buf);
    }

    self.cmd_bufs.push(WBarr::render().run_on_new_cmd_buff(w_v));
  }
}
