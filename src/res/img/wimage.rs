use std::mem::MaybeUninit;

use ash::vk;

use gpu_alloc::GpuAllocator;
use gpu_alloc_ash::AshMemoryDevice;
use smallvec::SmallVec;

use crate::{
  res::wbindings::WBindingAttachmentTrait,
  sys::{warenaitems::WAIdxImage, wdevice::WDevice},
};

#[derive(Clone)]
pub struct WImageInfo {
  pub resx: u32,
  pub resy: u32,
  pub resz: u32,
  pub format: vk::Format,
  pub is_depth: bool,
  pub is_cubemap: bool,
  pub mip_levels: u32,
  pub usage_flags: vk::ImageUsageFlags,
  pub file_path: Option<String>,
  pub raw_pixels: Option<*mut u8>,
}

impl Default for WImageInfo {
  fn default() -> Self {
    Self {
      resx: 500,
      resy: 500,
      resz: 1,
      format: vk::Format::R16G16B16A16_UNORM,
      is_depth: false,
      file_path: None,
      usage_flags: vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::TRANSFER_SRC
        | vk::ImageUsageFlags::SAMPLED
        | vk::ImageUsageFlags::STORAGE
        | vk::ImageUsageFlags::COLOR_ATTACHMENT,
      raw_pixels: None,
      mip_levels: 1,
      is_cubemap: false,
    }
  }
}

pub struct WImage {
  pub handle: vk::Image,

  pub arena_index: WAIdxImage,

  pub view: vk::ImageView,
  pub mip_views: SmallVec<[vk::ImageView; 10]>,

  pub resx: u32,
  pub resy: u32,
  pub mip_levels: u32,

  // pub chan_cnt: u32,
  // pub bits_per_chan: u32,

  pub is_depth: bool,
  pub is_cubemap: bool,

  pub layer_count: u32,

  pub format: vk::Format,
  pub descriptor_image_info: vk::DescriptorImageInfo,

  pub imgui_id: imgui::TextureId,

  image_aspect_flags: vk::ImageAspectFlags,
  pub usage_flags: vk::ImageUsageFlags,
}

impl WBindingAttachmentTrait for WImage {
  fn get_binding_type(&self) -> vk::DescriptorType {
    vk::DescriptorType::STORAGE_IMAGE
  }
}
impl WImage {
  pub fn new_from_swapchain_image(
    device: &ash::Device,
    _img: vk::Image,
    format: vk::SurfaceFormatKHR,
    resx: u32,
    resy: u32,
  ) -> Self {
    let mut img = WImage {
      view: unsafe { MaybeUninit::zeroed().assume_init() },
      resx: resx,
      resy: resy,
      handle: _img,
      format: format.format,
      is_depth: false,
      descriptor_image_info: vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build(),
      image_aspect_flags: vk::ImageAspectFlags::COLOR,
      usage_flags: vk::ImageUsageFlags::empty(),
      arena_index: wmemzeroed!(),
      mip_levels: 1,
      imgui_id: wmemzeroed!(),
      is_cubemap: false,
      layer_count: 1,
      mip_views: SmallVec::new(),
    };

    let view = Self::get_view(device, &mut img);

    img.view = view;

    img
  }


  pub fn new(
    device: &ash::Device,
    allocator: &mut GpuAllocator<vk::DeviceMemory>,
    format: vk::Format,
    resx: u32,
    resy: u32,
    resz: u32,
    mip_levels: u32,
    is_depth: bool,
    is_cubemap: bool,
    usage_flags: vk::ImageUsageFlags,
  ) -> Self {
    let mut flags = vk::ImageCreateFlags::empty();
    if is_cubemap {
      flags |= vk::ImageCreateFlags::CUBE_COMPATIBLE;
    }

    let image_info = vk::ImageCreateInfo::builder()
      .flags(flags)
      .image_type(vk::ImageType::TYPE_2D)
      .format(format)
      .extent(vk::Extent3D {
        width: resx,
        height: resy,
        depth: 1,
      })
      .mip_levels(mip_levels)
      .array_layers(
        if is_cubemap{ 6 } else { 1 }
      )
      .usage(usage_flags)
      .samples(vk::SampleCountFlags::TYPE_1)
      .tiling(vk::ImageTiling::OPTIMAL)
      .sharing_mode(vk::SharingMode::EXCLUSIVE)
      .initial_layout(vk::ImageLayout::UNDEFINED);

    let image_info = image_info.build();
    // VK_IMAGE_USAGE_STORAGE_BIT

    let image = unsafe { device.create_image(&image_info, None).unwrap() };

    let mem_req = unsafe { device.get_image_memory_requirements(image) };

    let block = unsafe {
      allocator
        .alloc(
          AshMemoryDevice::wrap(device),
          gpu_alloc::Request {
            size: mem_req.size,
            align_mask: mem_req.alignment - 1,
            usage: gpu_alloc::UsageFlags::FAST_DEVICE_ACCESS,
            // Todo: make this safer? or not give a shit
            memory_types: mem_req.memory_type_bits,
          },
        )
        .unwrap()
    };

    unsafe {
      device.bind_image_memory(image, *block.memory(), block.offset());
    }


    let mut img = WImage {
      view: unsafe { MaybeUninit::zeroed().assume_init() },
      resx,
      resy,
      handle: image,
      format,
      descriptor_image_info: wmemzeroed!(),
      is_depth,
      usage_flags,
      image_aspect_flags: if is_depth {
        vk::ImageAspectFlags::DEPTH
      } else {
        vk::ImageAspectFlags::COLOR
      },
      arena_index: wmemzeroed!(),
      mip_levels,
      imgui_id: wmemzeroed!(),
      is_cubemap,
      layer_count: if is_cubemap {6} else {1},
      mip_views: SmallVec::new(),
    };

    img.view = Self::get_view(device, &mut img);

    img.descriptor_image_info = vk::DescriptorImageInfo::builder()
      .image_layout(image_info.initial_layout)
      .image_view(img.view)
      .build();

    img
  }
  pub fn generate_mipmaps(
    &mut self,
    w_device: &mut WDevice,
  ) {
    unsafe {
      let layer_cnt = if self.is_cubemap {6} else {1};

      let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();

      let cmd_buf = w_device.curr_pool().get_cmd_buff();
      w_device
        .device
        .begin_command_buffer(cmd_buf, &cmd_buf_begin_info) ;

      //  layer here as well?
      let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_array_layer(0)
        .layer_count(1)
        .level_count(1);

      let mut barrier = vk::ImageMemoryBarrier::builder()
        .image(self.handle)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(*subresource);

      {
        let mut mip_width = self.resx;
        let mut mip_height = self.resy;

        for i in 1..self.mip_levels {
          barrier.subresource_range.base_mip_level = i - 1;
          barrier.old_layout = self.descriptor_image_info.image_layout;
          barrier.new_layout = self.descriptor_image_info.image_layout;
          barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
          barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

          w_device.device.cmd_pipeline_barrier(
            cmd_buf,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[*barrier],
          );

          let src_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(i - 1)
            .base_array_layer(0)
            .layer_count(layer_cnt);

          let dst_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(i)
            .base_array_layer(0)
            .layer_count(layer_cnt);

          let blit = vk::ImageBlit::builder()
            .src_offsets([
              vk::Offset3D { x: 0, y: 0, z: 0 },
              vk::Offset3D {
                x: mip_width as i32,
                y: mip_height as i32,
                z: 1,
              },
            ])
            .src_subresource(*src_subresource)
            .dst_offsets([
              vk::Offset3D { x: 0, y: 0, z: 0 },
              vk::Offset3D {
                x: (if mip_width > 1 { mip_width / 2 } else { 1 }) as i32,
                y: (if mip_height > 1 { mip_height / 2 } else { 1 }) as i32,
                z: 1,
              },
            ])
            .dst_subresource(*dst_subresource);

          w_device.device.cmd_blit_image(
            cmd_buf,
            self.handle,
            // vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
self.descriptor_image_info.image_layout,
            self.handle,
            // vk::ImageLayout::TRANSFER_DST_OPTIMAL,
self.descriptor_image_info.image_layout,
            &[*blit],
            vk::Filter::LINEAR,
          );

          barrier.old_layout = self.descriptor_image_info.image_layout;
          barrier.new_layout = self.descriptor_image_info.image_layout;
          barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
          barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

          w_device.device.cmd_pipeline_barrier(
            cmd_buf,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[] as &[vk::MemoryBarrier],
            &[] as &[vk::BufferMemoryBarrier],
            &[*barrier],
          );

          if mip_width > 1 {
            mip_width /= 2;
          }

          if mip_height > 1 {
            mip_height /= 2;
          }
        }
      }

      barrier.subresource_range.base_mip_level = self.mip_levels - 1;
      barrier.old_layout = self.descriptor_image_info.image_layout;
      barrier.new_layout = self.descriptor_image_info.image_layout;
      barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
      barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

      // finish up

      // device.cmd_pipeline_barrier2(
      //   cmd_buf,
      //   &*vk::DependencyInfo::builder().image_memory_barriers(&mem_bar),
      // );

      w_device.device.end_command_buffer(cmd_buf).unwrap();

      let mut cmd_buffs = [vk::CommandBufferSubmitInfo::builder()
        .command_buffer(cmd_buf)
        .build()];

      let submit_info = vk::SubmitInfo2::builder()
        .command_buffer_infos(&cmd_buffs)
        .build();

      w_device
        .device
        .queue_submit2(w_device.queue, &[submit_info], vk::Fence::null())
        .unwrap();

      w_device
        .device
        .queue_wait_idle(w_device.queue);

    }
  }

  pub fn change_layout(
    &mut self,
    w_device: &mut WDevice,
    new_layout: vk::ImageLayout,
    cmd_buf: vk::CommandBuffer,
  ) {
    let cmd_buf = w_device.single_command_begin();

    // should generalize this
    let subresource_range = vk::ImageSubresourceRange::builder()
      .aspect_mask(self.image_aspect_flags)
      .base_mip_level(0)
      .level_count(self.mip_levels)
      .base_array_layer(0)
      .layer_count(
        self.layer_count
      );

    let mem_bar = [vk::ImageMemoryBarrier2::builder()
      // .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
      .old_layout(self.descriptor_image_info.image_layout)
      .new_layout(new_layout)
      .src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
      .dst_stage_mask(vk::PipelineStageFlags2::TRANSFER)
      .image(self.handle)
      .subresource_range(*subresource_range)
      .build()];

    unsafe {
      w_device.device.cmd_pipeline_barrier2(
        cmd_buf,
        &*vk::DependencyInfo::builder().image_memory_barriers(&mem_bar),
      );
    }
    w_device.single_command_end_submit(cmd_buf);

    self.descriptor_image_info.image_layout = new_layout;

    self.view = Self::get_view(&w_device.device, self);
    self.descriptor_image_info.image_view = self.view;
  }

  fn get_view(
    device: &ash::Device,
    img: &mut WImage,
  ) -> vk::ImageView {
    // Todo: maybe not spam views if already created?
    let image_view_info = vk::ImageViewCreateInfo::builder()
      .image(img.handle)
      .view_type(
          if img.is_cubemap {
            vk::ImageViewType::CUBE
          } else {
            vk::ImageViewType::TYPE_2D
          }
        )
      .format(img.format)
      .components(vk::ComponentMapping {
        r: vk::ComponentSwizzle::IDENTITY,
        g: vk::ComponentSwizzle::IDENTITY,
        b: vk::ComponentSwizzle::IDENTITY,
        a: vk::ComponentSwizzle::IDENTITY,
      })
      .subresource_range(
        vk::ImageSubresourceRange::builder()
          .aspect_mask(img.image_aspect_flags)
          .base_mip_level(0)
          .level_count(img.mip_levels)
          .base_array_layer(0)
          .layer_count(img.layer_count)
          .build(),
      );
      
    if img.is_cubemap && img.mip_levels > 1 {
      unsafe{
        img.mip_views.set_len(0);
      }
      for mip_level in 0..img.mip_levels{
        let mip_view = vk::ImageViewCreateInfo::builder()
          .image(img.handle)
          .view_type(
              if img.is_cubemap {
                vk::ImageViewType::CUBE
              } else {
                vk::ImageViewType::TYPE_2D
              }
            )
          .format(img.format)
          .components(vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
          })
          .subresource_range(
            vk::ImageSubresourceRange::builder()
              .aspect_mask(img.image_aspect_flags)
              .base_mip_level(mip_level)
              .level_count(1)
              .base_array_layer(0)
              .layer_count(img.layer_count)
              .build(),
          ).build();
        let mip_view = unsafe { device.create_image_view(&mip_view, None).unwrap() };
        img.mip_views.push(mip_view);
      }
    }

    let view = unsafe { device.create_image_view(&image_view_info, None).unwrap() };

    view
  }

  fn build(mut self) -> Self {
    self
  }
}

// impl Default for WImage{
//     fn default() -> Self {
//         Self { handle: None, resx: 500, resy: 500, format: None, view: None }
//     }
// }
