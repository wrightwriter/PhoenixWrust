use ash::vk::{self, Rect2D, Offset2D, Extent2D};

use crate::{sys::{warenaitems::{WAIdxImage, WArenaItem}, wtl::WTechLead}, wvulkan::WVulkan, res::img::{wimage::WImageInfo, wrendertarget::{WRTInfo, WRPConfig}}};

use super::wthingnull::WThingNull;



pub struct WIbl{
  pub hdri: WAIdxImage,
  pub cubemap: WAIdxImage,
  pub cubemap_irradiance: WAIdxImage,
  pub cubemap_prefilter: WAIdxImage,
}

impl WIbl {
  pub fn new<S: Into<String>>(
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    file_name: S, 
  ) -> Self {
    let mut hdri_info = WImageInfo {  
      file_path: Some(file_name.into()), 
      ..wdef!()
    };

    hdri_info.format = vk::Format::R32G32B32A32_SFLOAT;

    hdri_info.usage_flags = vk::ImageUsageFlags::TRANSFER_DST
      | vk::ImageUsageFlags::TRANSFER_SRC
      | vk::ImageUsageFlags::SAMPLED
      | vk::ImageUsageFlags::STORAGE
      | vk::ImageUsageFlags::COLOR_ATTACHMENT;

    let hdri = w_tl.new_image(
        w_v, hdri_info.clone()
      ).0;

    {
      let shader_prog = w_v
        .w_shader_man
        .new_render_program(&mut w_v.w_device, "cubemap.vert", "cubemap.frag");
      let mut cubemap_info = hdri_info.clone();
        cubemap_info.resx = 512;
        cubemap_info.resy = 512;
        cubemap_info.is_cubemap = true;
        cubemap_info.mip_levels = 6;
        cubemap_info.file_path = None;

      let cubemap = w_tl.new_image(w_v,  cubemap_info.clone()).0;
      

      cubemap_info.mip_levels = 1;
      cubemap_info.resx = 32;
      cubemap_info.resy = 32;
      let cubemap_irradiance = w_tl.new_image(w_v,  cubemap_info.clone()).0;


      cubemap_info.resx = 128;
      cubemap_info.resy = 128;
      cubemap_info.mip_levels = 6;
      let cubemap_prefilter = w_tl.new_image(w_v,  cubemap_info.clone()).0;


      let mut thing = WThingNull::new(w_v, w_tl, shader_prog);
      
      
      // -- Draw cubemap -- //
      let mut rt = w_tl.new_render_target(w_v, WRTInfo::from_images(&[cubemap])).0;

      unsafe {
        let cmd_buf = rt.get_mut().begin_pass_ext(&mut w_v.w_device, WRPConfig{ layer_cnt: 6, ..wdef!() });
        
        thing.push_constants.reset();
        thing.push_constants.add(hdri);
        thing.push_constants.add(0 as u8);
        
        // thing.push_constants.add(hdr_img_idx);
        
        thing.draw_cnt(w_v, w_tl, Some(rt), &cmd_buf,4,6);

        rt.get_mut().end_pass(&mut w_v.w_device);
        
        w_v.w_device.single_command_submit(cmd_buf);
        w_v.w_device.device.queue_wait_idle(w_v.w_device.queue);
      }

      cubemap.get_mut().generate_mipmaps(&mut w_v.w_device);

      thing.rt = None;
      // -- Blurred cubemap -- //
      let mut rt = w_tl.new_render_target(w_v, WRTInfo::from_images(&[cubemap_irradiance])).0;

      unsafe {
        let cmd_buf = rt.get_mut().begin_pass_ext(&mut w_v.w_device, WRPConfig{ layer_cnt: 6, ..wdef!() });
        
        thing.push_constants.reset();
        thing.push_constants.add(cubemap);
        thing.push_constants.add(1 as u8);
        
        // thing.push_constants.add(hdr_img_idx);
        
        thing.draw_cnt(w_v, w_tl, Some(rt), &cmd_buf,4,6);

        rt.get_mut().end_pass(&mut w_v.w_device);
        
        w_v.w_device.single_command_submit(cmd_buf);
        w_v.w_device.device.queue_wait_idle(w_v.w_device.queue);
      }
      
      let mip_views = cubemap_prefilter.get().mip_views.clone();
      let mip_cnt = mip_views.len();

      // -- Prefiltered cubemap -- //
      let mut rt = w_tl.new_render_target(w_v, WRTInfo::from_images(&[cubemap_prefilter])).0;

      
      let mut i = 0;
      for view in mip_views{
        unsafe {
          let attachment = 
            vk::RenderingAttachmentInfo::builder()
              .image_view(view)
              .image_layout(vk::ImageLayout::GENERAL)
              .load_op(vk::AttachmentLoadOp::CLEAR)
              .store_op(vk::AttachmentStoreOp::STORE)
              .clear_value(vk::ClearValue {
                color: vk::ClearColorValue{
                  float32: [0.,0.,0.,0.],
                },
              }).build();

          let mut viewport = vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width((cubemap_prefilter.get().resx) as f32)
                .height((cubemap_prefilter.get().resy) as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .build();
          for k in 0..i{
            viewport.width = ((viewport.width as u32)/2 ) as f32;
            viewport.height= ((viewport.height as u32)/2 ) as f32;
          }
          
          // let mut render_area = Rect2D{ offset: Offset2D, extent: todo!() };
          let mut render_area = Rect2D::builder()
            .extent(Extent2D{ width: viewport.width as u32, height: viewport.height as u32 })
            .offset(Offset2D { x: 0, y: 0})
            .build()
          ;

          let cmd_buf = rt.get_mut().begin_pass_ext(&mut w_v.w_device, 
          WRPConfig{ layer_cnt: 6,  custom_attachments: Some(vec![ attachment ]), render_area: Some(render_area) });


          w_v.w_device
            .device
            .cmd_set_viewport(cmd_buf, 0, &[
              viewport
            ]);
          thing.rt = None;

          
          thing.push_constants.reset();
          thing.push_constants.add(cubemap);
          thing.push_constants.add(2 as u8);
          thing.push_constants.add(i as f32 / ((mip_cnt-1) as f32));

          
          thing.draw_cnt(w_v, w_tl, Some(rt), &cmd_buf,4,6);

          rt.get_mut().end_pass(&mut w_v.w_device);
          
          w_v.w_device.single_command_submit(cmd_buf);
          w_v.w_device.device.queue_wait_idle(w_v.w_device.queue);
        }

        i += 1;

      }

      Self { hdri, cubemap, cubemap_irradiance, cubemap_prefilter }
    }
  }

}





