// #![allow(unused)]
// #![allow(unused_imports)]
// #![allow(dead_code)]
// #![allow(non_upper_case_globals)]
// #![allow(invalid_value)]

extern crate spirv_reflect;

use ash::{
  extensions::{
    khr::{ Surface, Swapchain},
  },
  vk::{
    self,
    // vk::{
    SwapchainKHR,
  },
};

// use regex::Regex;




use winit::{
  dpi::PhysicalSize,
  window::Window,
};


use crate::res::{wimage::WImage, wrendertarget::WRenderTarget};

pub struct WSwapchain {
  pub swapchain: SwapchainKHR,
  pub swapchain_loader: Swapchain,
  pub swapchain_images_vk: Vec<vk::Image>,
  pub default_render_targets: Vec<WRenderTarget>,
  pub surface_format: vk::SurfaceFormatKHR,
  pub width: u32,
  pub height: u32,
  pub image_available_semaphores: Vec<vk::Semaphore>,
  pub render_finished_semaphores: Vec<vk::Semaphore>,
  pub in_flight_fences: Vec<vk::Fence>,
}

impl WSwapchain {
  pub fn new(
    device: &ash::Device,
    physical_device: &vk::PhysicalDevice,
    instance: &ash::Instance,
    surface_loader: &Surface,
    surface: &vk::SurfaceKHR,
    surface_format: &vk::SurfaceFormatKHR,
    present_mode: &vk::PresentModeKHR,
    // command_pool: &vk::CommandPool,
    window: &Window,
    #[allow(non_snake_case)]
    FRAMES_IN_FLIGHT: usize,
  ) -> Self {
    // https://vulkan-tutorial.com/Drawing_a_triangle/Presentation/Swap_chain
    let surface_caps = unsafe {
      surface_loader.get_physical_device_surface_capabilities(*physical_device, *surface)
    }
    .unwrap();

    let width = surface_caps.current_extent.width;
    let height = surface_caps.current_extent.height;

    let mut image_count = surface_caps.min_image_count + 1;
    if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
      image_count = surface_caps.max_image_count;
    }

    let extent = vk::Extent2D {
      width: width,
      height: height,
    };
    // https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/VkSurfaceCapabilitiesKHR.html#_description
    let swapchain_image_extent = match surface_caps.current_extent {
      extent => {
        let PhysicalSize { width, height } = window.inner_size();
        extent
      }
      #[allow(unreachable_patterns)]
      normal => normal,
    };

    let swapchain_info = vk::SwapchainCreateInfoKHR::builder()
      .surface(*surface)
      .min_image_count(image_count)
      .image_format(surface_format.format)
      .image_color_space(surface_format.color_space)
      .image_extent(swapchain_image_extent)
      .image_array_layers(1)
      .image_usage(
        vk::ImageUsageFlags::COLOR_ATTACHMENT |
        vk::ImageUsageFlags::TRANSFER_DST
      )
      .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
      .pre_transform(surface_caps.current_transform)
      // .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE_KHR)
      .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
      .present_mode(*present_mode)
      .clipped(true)
      .old_swapchain(vk::SwapchainKHR::null());

    let swapchain_loader = Swapchain::new(instance, device);

    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_info, None) }.unwrap();

    // !! ---------- Swapchain FBs ---------- //
    // https://vulkan-tutorial.com/Drawing_a_triangle/Drawing/Framebuffers

    let swapchain_images_vk = unsafe { swapchain_loader.get_swapchain_images(swapchain) }.unwrap();

    // let mut swapchain_images: Vec<WImage> = swapchain_images_vk
    let mut default_render_targets: Vec<WRenderTarget> = swapchain_images_vk
      .iter()
      .map(|image| {
        let swapchain_image = WImage::new_from_swapchain_image(&*device, *image, *surface_format, width, height);

        WRenderTarget::new_from_swapchain(device, *surface_format, vec![swapchain_image])
      })
      .collect();

    // let mut default_render_targets: Vec<WRenderTarget> = swapchain_images
    //   .iter()
    //   .map(|swapchain_image| {
    //     // let mut img: WImage = unsafe { (swapchain_image as *const WImage) };
    //     // let mut img: WImage = WImage;
    //     WRenderTarget::new_from_swapchain(device, *surface_format, vec![*swapchain_image])
    //   })
    //   .collect();

    // !! ---------- Semaphores ---------- //

    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let image_available_semaphores: Vec<_> = (0..FRAMES_IN_FLIGHT)
      .map(|_| unsafe { device.create_semaphore(&semaphore_info, None) }.unwrap())
      .collect();
    let render_finished_semaphores: Vec<_> = (0..FRAMES_IN_FLIGHT)
      .map(|_| unsafe { device.create_semaphore(&semaphore_info, None) }.unwrap())
      .collect();

    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    let in_flight_fences: Vec<_> = (0..FRAMES_IN_FLIGHT)
      .map(|_| unsafe { device.create_fence(&fence_info, None) }.unwrap())
      .collect();

    Self {
      swapchain,
      swapchain_loader,
      swapchain_images_vk,
      surface_format: *surface_format,
      width,
      height,
      default_render_targets,
      image_available_semaphores,
      render_finished_semaphores,
      in_flight_fences,
    }
  }
}
