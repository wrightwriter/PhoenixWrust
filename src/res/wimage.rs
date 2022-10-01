use std::mem::MaybeUninit;

use ash::vk;

use getset::Getters;
use gpu_alloc::GpuAllocator;
use gpu_alloc_ash::AshMemoryDevice;

use crate::{
  res::wbindings::WBindingAttachmentTrait,
  sys::wdevice::WDevice,
  sys::wmanagers::{WAIdxImage, WTechLead},
};

// !! ---------- IMAGE ---------- //

#[derive(Getters)]
pub struct WImage {
  pub handle: vk::Image,
  #[getset(get = "pub")]
  view: vk::ImageView,
  #[getset(get = "pub")]
  resx: u32,
  #[getset(get = "pub")]
  resy: u32,
  pub format: vk::Format,
  pub descriptor_image_info: vk::DescriptorImageInfo,
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
      descriptor_image_info: wmemzeroed!(),
    };

    let view = Self::get_view(device, &img);

    img.view = view;

    img
  }

  pub fn change_layout(
    &mut self,
    w_device: &mut WDevice,
    new_layout: vk::ImageLayout,
  ) {
    let device = &mut w_device.device;

    let cmd_buf_allocate_info = vk::CommandBufferAllocateInfo::builder()
      .command_pool(w_device.command_pool)
      .level(vk::CommandBufferLevel::PRIMARY)
      .command_buffer_count(1);

    let command_buffer =
      unsafe { device.allocate_command_buffers(&cmd_buf_allocate_info) }.unwrap()[0];

    let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();

    unsafe {
      device
        .begin_command_buffer(command_buffer, &cmd_buf_begin_info)
        .unwrap();
    }

    // should generalize this
    let subresource_range = vk::ImageSubresourceRange::builder()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_mip_level(0)
      .level_count(1)
      .base_array_layer(0)
      .layer_count(1);

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
      device.cmd_pipeline_barrier2(
        command_buffer,
        &*vk::DependencyInfo::builder().image_memory_barriers(&mem_bar),
      );
      device.end_command_buffer(command_buffer).unwrap();

      let mut cmd_buffs = [
        vk::CommandBufferSubmitInfo::builder()
          .command_buffer(command_buffer)
          .build()];

      let submit_info = vk::SubmitInfo2::builder()
        .command_buffer_infos(&cmd_buffs)
        .build();


      device
        .queue_submit2(w_device.queue, &[submit_info], vk::Fence::null())
        .unwrap();
    }

    self.descriptor_image_info.image_layout = new_layout;

    self.view = Self::get_view(device, &self);
    self.descriptor_image_info.image_view = self.view;
  }

  pub fn new(
    device: &ash::Device,
    allocator: &mut GpuAllocator<vk::DeviceMemory>,
    format: vk::Format,
    resx: u32,
    resy: u32,
    resz: u32,
  ) -> Self {
    let flags = vk::ImageCreateFlags::empty();

    let image_info = vk::ImageCreateInfo::builder()
      .flags(flags)
      .image_type(vk::ImageType::TYPE_2D)
      .format(format)
      .extent(vk::Extent3D {
        width: resx,
        height: resy,
        depth: 1,
      })
      .mip_levels(1)
      .array_layers(1)
      .usage(
        vk::ImageUsageFlags::TRANSFER_DST
          | vk::ImageUsageFlags::SAMPLED
          | vk::ImageUsageFlags::STORAGE
          | vk::ImageUsageFlags::COLOR_ATTACHMENT
          ,
      )
      .samples(vk::SampleCountFlags::TYPE_1)
      .tiling(vk::ImageTiling::OPTIMAL)
      .sharing_mode(vk::SharingMode::EXCLUSIVE)
      .initial_layout(vk::ImageLayout::UNDEFINED);

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

    let mut view = unsafe { MaybeUninit::zeroed().assume_init() };

    let mut img = WImage {
      view: view,
      resx: resx,
      resy: resy,
      handle: image,
      format: format,
      descriptor_image_info: wmemzeroed!(),
    };

    view = Self::get_view(device, &img);

    img.view = view;
    img.descriptor_image_info = vk::DescriptorImageInfo::builder()
      .image_layout(image_info.initial_layout)
      .image_view(img.view)
      .build();

    img
  }

  fn get_view(
    device: &ash::Device,
    img: &WImage,
  ) -> vk::ImageView {
    // Todo: maybe not spam views if already created?
    let image_view_info = vk::ImageViewCreateInfo::builder()
      .image(img.handle)
      .view_type(vk::ImageViewType::TYPE_2D)
      .format(img.format)
      .components(vk::ComponentMapping {
        r: vk::ComponentSwizzle::IDENTITY,
        g: vk::ComponentSwizzle::IDENTITY,
        b: vk::ComponentSwizzle::IDENTITY,
        a: vk::ComponentSwizzle::IDENTITY,
      })
      .subresource_range(
        vk::ImageSubresourceRange::builder()
          .aspect_mask(vk::ImageAspectFlags::COLOR)
          .base_mip_level(0)
          .level_count(1)
          .base_array_layer(0)
          .layer_count(1)
          .build(),
      );

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
