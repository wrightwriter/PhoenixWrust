// !! ---------- RENDERTARGET ---------- //

use std::ops::BitOr;

use ash::vk::{self, Rect2D, RenderingAttachmentInfo};
use derive_builder::Builder;
use smallvec::SmallVec;

use crate::{
  res::{img::wimage::WImage, wpongabletrait::WPongableTrait},
  sys::{
    warenaitems::{WAIdxImage, WArenaItem},
    wdevice::WDevice,
    wtl::WTechLead,
  },
  wvulkan::WVulkan,
};

use super::wimage::WImageInfo;

#[derive(Clone)]
pub struct WRTInfo {
  pub resx: u32,
  pub resy: u32,
  pub format: vk::Format,
  pub pongable: bool,
  // pub cnt_attachments: u64,
  pub attachment_infos: Vec<WImageInfo>,
  pub attachment_images: Option<SmallVec<[WAIdxImage; 10]>>,
  pub load_op: vk::AttachmentLoadOp,
  pub store_op: vk::AttachmentStoreOp,
  pub has_depth: bool,
}

impl WRTInfo {
  pub fn from_images(images: &[WAIdxImage]) -> Self {
    Self {
      attachment_images: Some(images.into()),
      ..wdef!()
    }
  }
}

impl Default for WRTInfo {
  fn default() -> Self {
    Self {
      resx: 500,
      resy: 500,
      pongable: false,
      format: vk::Format::R16G16B16A16_UNORM,
      // cnt_attachments: 1,
      attachment_infos: vec![WImageInfo { ..wdef!() }],
      attachment_images: None,
      load_op: vk::AttachmentLoadOp::CLEAR,
      store_op: vk::AttachmentStoreOp::STORE,
      has_depth: true,
    }
  }
}

impl WRTInfo {
  pub fn create(
    &self,
    // w_device: &mut WDevice,
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
  ) -> WRenderTarget {
    if let Some(images) = &self.attachment_images {
      WRenderTarget::new_from_images(w_v, w_tl, &images, None)
    } else {
      WRenderTarget::new(w_v, w_tl, self.clone())
    }
  }
}


#[derive(Builder, Debug)]
#[builder(setter(into))]
pub struct WRPConfig {
  pub layer_cnt: u32,
  pub custom_attachments: Option<Vec<vk::RenderingAttachmentInfo>>,
  pub render_area: Option<Rect2D>,
  pub load_op: Option<SmallVec<[vk::AttachmentLoadOp; 10]>>,
  pub store_op: Option<SmallVec<[vk::AttachmentStoreOp;10]>>,
  // pub res: Option<[u32; 2]>,
  // start_layer: u32,
}
impl Default for WRPConfig {
  fn default() -> Self {
    Self {
      layer_cnt: 1,
      custom_attachments: None,
      render_area: None,
      load_op: None,
      store_op: None,
    }
  }
}

pub struct WRenderTarget {
  pub images: Vec<WImage>,
  pub image_indices: [SmallVec<[WAIdxImage; 10]>; 2],
  pub image_depth: Option<WAIdxImage>,

  pub cmd_buf: vk::CommandBuffer,
  pub resx: u32,
  pub resy: u32,

  pub pongable: bool,
  pub pong_idx: u32,
  pub mem_bars_in: SmallVec<[vk::ImageMemoryBarrier2; 10]>,
  pub mem_bars_out: SmallVec<[vk::ImageMemoryBarrier2; 10]>,
  // pub clear_values: vec![vk::ClearValue {
  // pub clear_values: SmallVec::<[vk::ClearValue;10]>,
  // pub load_ops: SmallVec::<[vk::AttachmentLoadOp;10]>,
  // pub store_ops: SmallVec::<[vk::AttachmentStoreOp;10]>,
  pub attachment_infos: [SmallVec<[vk::RenderingAttachmentInfo; 10]>; 2],
  pub depth_attachment_info: Option<vk::RenderingAttachmentInfo>,
  pub render_area: vk::Rect2D,
}

impl WPongableTrait for WRenderTarget {
  fn pong(&mut self) {
    if self.pongable {
      self.pong_idx = 1 - self.pong_idx;
    }
  }

  fn is_pongable(&mut self) -> bool {
    self.pongable
  }
}

impl WRenderTarget {
  // pub fn get_images(&mut self) -> &SmallVec<[WAIdxImage; 10]> {
  //   &self.image_indices[self.pong_idx as usize]
  // }

  // fn create_images() {}

  pub fn image_at(
    &self,
    idx: usize,
  ) -> WAIdxImage {
    self.image_indices[self.pong_idx as usize][idx]
  }
  pub fn back_image_at(
    &self,
    idx: usize,
  ) -> WAIdxImage {
    self.image_indices[1 - self.pong_idx as usize][idx]
  }

  fn get_render_area(
    resx: u32,
    resy: u32,
  ) -> vk::Rect2D {
    vk::Rect2D {
      offset: vk::Offset2D { x: 0, y: 0 },
      extent: vk::Extent2D { width: resx, height: resy },
    }
  }

  pub fn new_from_images(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    images: &[WAIdxImage],
    depth: Option<WAIdxImage>,
  ) -> Self {
    let mut attachment_infos = [SmallVec::new(), SmallVec::new()];
    let mut image_indices = [SmallVec::new(), SmallVec::new()];

    // image_indices[0] = images.iter().map(|i|{*i}).collect();
    // image_indices[1] = images.iter().map(|i|{*i}).collect();
    
    debug_assert!(images.len() == 1);

    image_indices[0] = SmallVec::new();
    image_indices[0].push(images[0]);
    image_indices[1] = SmallVec::new();
    image_indices[1].push(images[0]);

    let mut resx: u32 = images[0].get().resx;
    let mut resy: u32 = images[0].get().resy;

    for image in images {
      let image = image.get();
      let attachment_info = vk::RenderingAttachmentInfo::builder()
        .image_view(image.view)
        .image_layout(vk::ImageLayout::GENERAL)
        // .load_op(clear)
        // .samples(vk::SampleCountFlags::_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        // .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        // .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        // .initial_layout(vk::ImageLayout::UNDEFINED)
        .clear_value(vk::ClearValue {
          // col
          color: vk::ClearColorValue { float32: [0., 0., 0., 0.] },
          // depth_stencil: vk::ClearDepthStencilValue {
          //   depth: 1.0,
          //   stencil: 0,
          // },
        })
        .build();
      attachment_infos[0].push(attachment_info);
      // image_indices[pong_idx].push(image.0);
    }

    // image_indices[0] = images.iter().map(|i|{i}).collect();
    Self {
      images: vec![],
      image_indices,
      image_depth: depth,
      cmd_buf: wmemzeroed!(),
      resx,
      resy,
      pongable: false,
      pong_idx: 0,
      mem_bars_in: SmallVec::new(),
      mem_bars_out: SmallVec::new(),
      attachment_infos,
      depth_attachment_info: None,
      render_area: Self::get_render_area(resx, resy),
    }
  }

  fn new(
    // w_device: &mut WDevice,
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    create_info: WRTInfo,
  ) -> Self {
    let pong_idx = 0;

    let WRTInfo {
      resx,
      resy,
      attachment_infos: attachments,
      format,
      pongable,
      has_depth: depth_attachment,
      ..
    } = create_info;

    let render_area = Self::get_render_area(resx, resy);

    let mut depth_attachment_info = None;
    let image_depth;

    if depth_attachment {
      let depth_image = w_tl.new_image(
        w_v,
        WImageInfo {
          resx,
          resy,
          resz: 1,
          format: vk::Format::D32_SFLOAT,
          is_depth: true,
          usage_flags: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                | vk::ImageUsageFlags::TRANSFER_DST
                // | vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::SAMPLED,
          ..wdef!()
        },
      );
      image_depth = Some(depth_image.0);
      let attachment_info = vk::RenderingAttachmentInfo::builder()
        .image_view(depth_image.1.view)
        .image_layout(vk::ImageLayout::GENERAL)
        // .load_op(clear)
        // .samples(vk::SampleCountFlags::_1)
        .load_op(create_info.load_op)
        .store_op(create_info.store_op)
        // .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        // .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        // .initial_layout(vk::ImageLayout::UNDEFINED)
        .clear_value(vk::ClearValue {
          depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
        })
        .build();
      depth_attachment_info = Some(attachment_info);
    } else {
      image_depth = None;
    }

    let mut rendering_attachment_infos = [SmallVec::new(), SmallVec::new()];
    let mut image_indices = [SmallVec::new(), SmallVec::new()];

    let pong_cnt = if pongable { 2 } else { 1 };

    for pong_idx in 0..pong_cnt {
      for attachment_info in attachments.clone() {
        let mut attachment_info = attachment_info;
        attachment_info.usage_flags = attachment_info.usage_flags.bitor(vk::ImageUsageFlags::TRANSFER_SRC);
        attachment_info.resx = resx;
        attachment_info.resy = resy;

        let image = w_tl.new_image(w_v, attachment_info);

        let attachment_info = vk::RenderingAttachmentInfo::builder()
          .image_view(image.1.view)
          .image_layout(vk::ImageLayout::GENERAL)
          // .load_op(clear)
          // .samples(vk::SampleCountFlags::_1)
          .load_op(create_info.load_op)
          .store_op(create_info.store_op)
          // .initial_layout(vk::ImageLayout::UNDEFINED)
          .clear_value(vk::ClearValue {
            color: vk::ClearColorValue {
              float32: [0.0, 0.0, 0.0, 1.0],
            },
          })
          .build();

        rendering_attachment_infos[pong_idx].push(attachment_info);
        image_indices[pong_idx].push(image.0);
      }

      // for attachment_idx in 0..cnt_attachments as usize {
      //   let image = w_tl.new_image(
      //     w_device,
      //     WImageCreateInfo {
      //       resx,
      //       resy,
      //       resz: 1,
      //       format: format,
      //       usage_flags: WImageCreateInfo::default().usage_flags.bitor(
      //         vk::ImageUsageFlags::TRANSFER_SRC
      //       ),
      //       ..wdef!()
      //     },
      //   );

      //   let attachment_info = vk::RenderingAttachmentInfo::builder()
      //     .image_view(image.1.view)
      //     .image_layout(vk::ImageLayout::GENERAL)
      //     // .load_op(clear)
      //     // .samples(vk::SampleCountFlags::_1)
      //     .load_op(create_info.load_op)
      //     .store_op(create_info.store_op)
      //     // .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
      //     // .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
      //     // .initial_layout(vk::ImageLayout::UNDEFINED)
      //     .clear_value(vk::ClearValue {
      //       color: vk::ClearColorValue {
      //         float32: [0.0, 0.0, 0.0, 1.0],
      //       },
      //     })
      //     .build();

      //   rendering_attachment_infos[pong_idx].push(attachment_info);
      //   image_indices[pong_idx].push(image.0);
      // }
    }

    Self {
      pong_idx,
      pongable,
      resx,
      resy,
      render_area,
      images: wmemzeroed!(),
      image_indices,
      image_depth,
      // render_pass: todo!(),
      // command_buffers,
      cmd_buf: wmemzeroed!(),
      attachment_infos: rendering_attachment_infos,
      mem_bars_in: SmallVec::new(),
      mem_bars_out: SmallVec::new(),
      depth_attachment_info,
    }
  }
  pub fn new_from_swapchain(
    device: &ash::Device,
    // command_pool: &CommandPool,
    format: vk::SurfaceFormatKHR,
    images: Vec<WImage>,
  ) -> Self {
    let pong_idx = 0;
    // let images_copy = images.clone();
    let images_copy = images;

    let subresource_range = vk::ImageSubresourceRange::builder()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_mip_level(0)
      .level_count(1)
      .base_array_layer(0)
      .layer_count(1);

    let mut mem_bars_in = SmallVec::new();
    mem_bars_in.push(
      vk::ImageMemoryBarrier2::builder()
        .dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
        .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
        .image(images_copy[0].handle)
        .subresource_range(*subresource_range)
        .build(),
    );

    let subresource_range = vk::ImageSubresourceRange::builder()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_mip_level(0)
      .level_count(1)
      .base_array_layer(0)
      .layer_count(1);

    let mut mem_bars_out = SmallVec::new();
    mem_bars_out.push(
      vk::ImageMemoryBarrier2::builder()
        .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
        .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
        .dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)
        .image(images_copy[0].handle)
        .subresource_range(*subresource_range)
        .build(),
    );

    let resx = images_copy[0].resx;
    let resy = images_copy[0].resy;

    let render_area = Self::get_render_area(resx, resy);

    let mut rendering_attachment_infos = [SmallVec::new(), SmallVec::new()];
    let image_indices = [SmallVec::new(), SmallVec::new()];

    let pongable = false;
    let pong_cnt = 1;

    for pong_idx in 0..pong_cnt {
      for attachment_idx in 0..1 as usize {
        let attachment_info = vk::RenderingAttachmentInfo::builder()
          .image_view(images_copy[0].view)
          .image_layout(vk::ImageLayout::GENERAL)
          // .load_op(clear)
          // .samples(vk::SampleCountFlags::_1)
          // .load_op(vk::AttachmentLoadOp::CLEAR)
          .load_op(vk::AttachmentLoadOp::LOAD)
          .store_op(vk::AttachmentStoreOp::STORE)
          // .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
          // .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
          // .initial_layout(vk::ImageLayout::UNDEFINED)
          .clear_value(vk::ClearValue {
            color: vk::ClearColorValue {
              float32: [0.0, 0.0, 0.0, 1.0],
            },
          })
          .build();
        rendering_attachment_infos[pong_idx].push(attachment_info);
      }
    }

    WRenderTarget {
      pong_idx,
      pongable,
      images: images_copy,
      cmd_buf: wmemzeroed!(),
      resx,
      resy,
      image_indices,
      attachment_infos: rendering_attachment_infos,
      render_area,
      mem_bars_in,
      mem_bars_out,
      image_depth: None,
      depth_attachment_info: None,
    }
  }
  pub fn begin_pass_ext(
    &mut self,
    w_device: &mut WDevice,
    config: WRPConfig,
  ) -> vk::CommandBuffer {
    self.cmd_buf = w_device.curr_pool().get_cmd_buff();

    let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();

    let mut render_area = if let Some(ra) = config.render_area {
      ra
    } else {
      self.render_area.clone()
    };
    // render_area.extent.width = res[0] as u32;
    // render_area.extent.height = res[1] as u32;

    let res = [render_area.extent.width as f32, render_area.extent.height as f32];
    // let res = if let Some(cfg_res) = config.res {
    //   [cfg_res[0] as f32, cfg_res[1] as f32]
    // } else {
    //   [self.resx as f32, self.resy as f32]
    // };
    // if config.res.is_some(){
    //   render_area.offset.x = render_area.extent.width as i32;
    //   render_area.offset.y = render_area.extent.height as i32;
    // }
    // let render_area = if config.res.is_some(){ res} else {[self.resx, self.resy]};

    unsafe {
      w_device.device.begin_command_buffer(self.cmd_buf, &cmd_buf_begin_info);
    }

    if self.mem_bars_in.len() > 0 {
      unsafe {
        w_device.device.cmd_pipeline_barrier2(
          self.cmd_buf,
          &*vk::DependencyInfo::builder().image_memory_barriers(&self.mem_bars_in),
        );
      }
    }

    let render_pong_idx;
    if self.pongable {
      render_pong_idx = self.pong_idx as usize;
    } else {
      render_pong_idx = 0;
    }

    if let Some(attachments) = &config.custom_attachments {
      unsafe {
        self.attachment_infos[0].set_len(0);
        for attachment in attachments {
          self.attachment_infos[0].push(*attachment)
        }
      }
    }
    
    let mut attachments = self.attachment_infos[render_pong_idx].clone();

    if let Some(load_ops) = &config.load_op {
      let mut i = 0;
      for att in &mut attachments{
        att.load_op = load_ops[i];
        i += 1;
      }
    }

    if let Some(store_ops) = &config.store_op {
      let mut i = 0;
      for att in &mut attachments{
        att.store_op = store_ops[i];
        i += 1;
      }
    }

    let mut rendering_info = vk::RenderingInfo::builder()
      .layer_count(config.layer_cnt)
      // .layer_count(6)
      .color_attachments(&attachments)
      .render_area(render_area)
      .build();

    if let Some(depth_attachment_info) = &self.depth_attachment_info {
      rendering_info.p_depth_attachment = depth_attachment_info;
    }

    unsafe {
      w_device.device.cmd_begin_rendering(self.cmd_buf, &rendering_info);
    }
    unsafe {
      w_device.device.cmd_set_viewport(
        self.cmd_buf,
        0,
        &[vk::Viewport::builder()
          .x(0.0)
          .y(0.0)
          .width(res[0])
          .height(res[1])
          .min_depth(0.0)
          .max_depth(1.0)
          .build()],
      );
      w_device.device.cmd_set_scissor(
        self.cmd_buf,
        0,
        &[self.render_area],
      )
    }
    return self.cmd_buf;
  }

  pub fn begin_pass(
    &mut self,
    w_device: &mut WDevice,
  ) -> vk::CommandBuffer {
    self.begin_pass_ext(w_device, WRPConfig::default())
  }
  pub fn end_pass(
    &mut self,
    w_device: &WDevice,
  ) -> vk::CommandBuffer {
    // let cmd_buf = &self.command_buffers[self.pong_idx as usize];
    let cmd_buf = &self.cmd_buf;
    unsafe {
      w_device.device.cmd_end_rendering(*cmd_buf);

      if self.mem_bars_out.len() > 0 {
        unsafe {
          w_device
            .device
            .cmd_pipeline_barrier2(*cmd_buf, &*vk::DependencyInfo::builder().image_memory_barriers(&self.mem_bars_out));
        }
      }

      w_device.device.end_command_buffer(*cmd_buf).unwrap();
    }
    self.cmd_buf
  }
}

// vk::SampleCountFlags::TYPE_1

// let attachments = vk::AttachmentDescription::builder();

// let attachments = vec![
//   vk::AttachmentDescription::builder()
//     .format(format.format)
//     .samples(vk::SampleCountFlags::TYPE_1)
//     .load_op(vk::AttachmentLoadOp::CLEAR)
//     .store_op(vk::AttachmentStoreOp::STORE)
//     .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
//     .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
//     .initial_layout(vk::ImageLayout::UNDEFINED)
//     .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
//     .build()
// ];

// let color_attachment_refs = vec![vk::AttachmentReference::builder()
//   .attachment(0)
//   .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
//   .build()];

// let subpasses = vec![vk::SubpassDescription::builder()
//   .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
//   .color_attachments(&color_attachment_refs)
//   .build()];

// let dependencies = vec![vk::SubpassDependency::builder()
//   .src_subpass(vk::SUBPASS_EXTERNAL)
//   .dst_subpass(0)
//   .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
//   .src_access_mask(vk::AccessFlags::empty())
//   .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
//   .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
//   .build()];

// let render_pass_info = vk::RenderPassCreateInfo::builder()
//   .attachments(&attachments)
//   .subpasses(&subpasses)
//   .dependencies(&dependencies);

// let render_pass = unsafe { device.create_render_pass(&render_pass_info, None) }.unwrap();

// let image_views: Vec<ImageView> = images.iter().map(|image|{image.view.unwrap()}).collect();

// let framebuffers: Vec<Framebuffer> = images
//   .iter()
//   .map(|image| {
//     // swapchain_images.push(image_view);

//     // let attachments = vec![*image_view];
//     let attachments = vec![*image.view()];

//     let framebuffer_info = vk::FramebufferCreateInfo::builder()
//       .render_pass(render_pass)
//       .attachments(&attachments)
//       .width(*image.resx())
//       .height(*image.resy())
//       .layers(1);

//     unsafe { device.create_framebuffer(&framebuffer_info, None) }.unwrap()
//   })
//   .collect();
// framebuffers: Vec<Framebuffer>,
// render_pass: vk::RenderPass,
