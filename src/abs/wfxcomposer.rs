use std::collections::HashMap;

use ash::vk;
use lazy_static::__Deref;
use smallvec::SmallVec;

use crate::{
  res::{
    buff::{
      wpushconstant::WPushConstant, wuniformscontainer::WUniformsContainer,
      wwritablebuffertrait::WWritableBufferTrait,
    },
    img::{
      wimage::WImageInfo,
      wrendertarget::{WRenderTarget, WRenderTargetInfo},
    },
    wpongabletrait::WPongableTrait,
    wshader::WShaderEnumPipelineBind,
  },
  sys::{
    warenaitems::{
      WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxRenderPipeline, WAIdxRt, WAIdxShaderProgram,
      WAIdxUbo, WArenaItem,
    },
    wbarr::WBarr,
    wdevice::{WDevice, GLOBALS},
    wmanagers::WGrouper,
    wrenderpipeline::{WRenderPipeline, WRenderPipelineTrait},
  },
  wvulkan::WVulkan,
};

use super::wpostpass::WPassTrait;

pub struct WFxComposer {
  pub rt: WAIdxRt,
  rt_in: WAIdxRt,
  pub cmd_bufs: SmallVec<[vk::CommandBuffer; 31]>,
}

impl WFxComposer {
  pub fn new(w: &mut WVulkan) -> Self {
    let rt_info = WRenderTargetInfo {
      resx: w.w_cam.width,
      resy: w.w_cam.height,
      pongable: true,
      has_depth: false,
      attachments: vec![WImageInfo { ..wdef!() }],
      ..wdef!()
    };
    let rt = w.w_tl.new_render_target(&mut w.w_device, rt_info);

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
    fx_pass: &mut impl WPassTrait,
  ) {
    let rt = self.rt.get_mut();
    rt.pong();

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
    fx_pass.run(w_v, &cmd_buf);
    rt.end_pass(&mut w_v.w_device);
    self.cmd_bufs.push(rt.cmd_buf);


    let bar_cmd_buf = w_v.w_device.curr_pool().get_cmd_buff();

    let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();
    unsafe {
      w_v
        .w_device
        .device
        .begin_command_buffer(bar_cmd_buf, &cmd_buf_begin_info)
        .unwrap();
    }
    let bar = WBarr::render().run_on_cmd_buff(&w_v.w_device, bar_cmd_buf);

    unsafe {
      w_v.w_device.device.end_command_buffer(bar_cmd_buf).unwrap();
    }

    self.cmd_bufs.push(bar_cmd_buf);
  }
}
