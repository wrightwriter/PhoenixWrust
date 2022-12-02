use ash::{
  vk::{
    self,
  },
};



use crate::{
  sys::{
    wdevice::WDevice,
  }, wvulkan::WVulkan,
};



pub type VStage = vk::PipelineStageFlags2;

#[derive(Clone, Copy)]
enum BarrierType {
  Image(vk::ImageMemoryBarrier2),
  General(vk::MemoryBarrier2),
  Buffer(vk::BufferMemoryBarrier2),
}
#[derive(Clone, Copy)]
pub struct WBarr {
  barrier: BarrierType,
}
impl WBarr {
  pub fn run_on_new_cmd_buff(
    &mut self,
    w_v: &mut WVulkan,
  ) -> vk::CommandBuffer{
    let w_device = &mut w_v.w_device;
    let cmd_buff = w_device.curr_pool().get_cmd_buff();

    // TODO: not do this lmao
    unsafe {
      let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();
      w_device
        .device
        .begin_command_buffer(cmd_buff, &cmd_buf_begin_info);

      self.run_on_cmd_buff(w_device, cmd_buff);

      w_device.device.end_command_buffer(cmd_buff);
    }
    cmd_buff
  }
  pub fn run_on_cmd_buff(
    &self,
    w_device: &WDevice,
    command_buffer: vk::CommandBuffer,
  ) -> WBarr {
    unsafe {
      match &self.barrier {
        BarrierType::Image(__) => {
          let mem_bar = [*__];
          let dep = vk::DependencyInfo::builder()
            .image_memory_barriers(&mem_bar)
            .build();
          w_device.device.cmd_pipeline_barrier2(command_buffer, &dep);
        }
        BarrierType::General(mut __) => {
          let mem_bar = [__];

          let dep = vk::DependencyInfo::builder()
            .memory_barriers(&mem_bar)

            // .dependency_flags(vk::DependencyFlags::)
            .build();
          // wprint!(dep.memory_barrier_count);
          w_device.device.cmd_pipeline_barrier2(command_buffer, &dep);
        }
        BarrierType::Buffer(__) => {
          // let mem_bar = [ &*vk::DependencyInfo::builder().buffer_memory_barriers(__).build()],
          let mem_bar = [*__];
          let dep = vk::DependencyInfo::builder()
            .buffer_memory_barriers(&mem_bar)
            .build();
          w_device.device.cmd_pipeline_barrier2(command_buffer, &dep);
        }
      }
    };
    *self
  }
  pub fn image() -> WBarr {
    let subresource_range = vk::ImageSubresourceRange::builder()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_mip_level(0)
      .level_count(1)
      .base_array_layer(0)
      .layer_count(1)
      .build();
    let barrier = BarrierType::Image(
      vk::ImageMemoryBarrier2::builder()
        .subresource_range(subresource_range)
        .build(),
    );
    WBarr { barrier }
  }
  pub fn comp_to_frag() -> WBarr {
    let mut b = WBarr {
      barrier: BarrierType::General(vk::MemoryBarrier2::builder()
        .build()),
    };
    b = b
      .src_stage(VStage::COMPUTE_SHADER)
      .dst_stage(VStage::FRAGMENT_SHADER)
      .dst_access(vk::AccessFlags2::SHADER_READ);
    b
  }
  pub fn render() -> WBarr {
    let mut b = WBarr {
      barrier: BarrierType::General(vk::MemoryBarrier2::builder()
        .build()),
    };
    b = b
      .src_stage(VStage::COLOR_ATTACHMENT_OUTPUT)
      .dst_stage(VStage::FRAGMENT_SHADER)
      .dst_access(vk::AccessFlags2::SHADER_READ);
    b
  }
  pub fn general() -> WBarr {
    let mut b = WBarr {
      barrier: BarrierType::General(vk::MemoryBarrier2::builder()
        .build()),
    };
    b = b
      .src_stage(VStage::BOTTOM_OF_PIPE)
      .dst_stage(VStage::TOP_OF_PIPE)
      .dst_access(vk::AccessFlags2::SHADER_READ);
      ;
    
    b
  }
  pub fn new_buffer_barr() -> WBarr {
    WBarr {
      barrier: BarrierType::Buffer(vk::BufferMemoryBarrier2::builder().build()),
    }
  }
  pub fn old_layout(
    &mut self,
    layout: vk::ImageLayout,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.old_layout = layout;
      }
      BarrierType::General(_) => {}
      BarrierType::Buffer(_) => {}
    };
    *self
  }
  pub fn new_layout(
    &mut self,
    layout: vk::ImageLayout,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.new_layout = layout;
      }
      BarrierType::General(_) => {}
      BarrierType::Buffer(_) => {}
    };
    *self
  }
  pub fn set_image(&mut self, image: vk::Image )->WBarr{
    match &mut self.barrier {
      BarrierType::Image(__) => {__.image = image;},
      BarrierType::General(_) => {},
      BarrierType::Buffer(_) => {},
    };
    *self
  }
  pub fn src_stage(
    &mut self,
    stage: vk::PipelineStageFlags2,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.src_stage_mask = stage;
      }
      BarrierType::General(__) => {
        __.src_stage_mask = stage;
      }
      BarrierType::Buffer(__) => {
        __.src_stage_mask = stage;
      }
    };
    *self
  }
  pub fn dst_stage(
    &mut self,
    stage: vk::PipelineStageFlags2,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.dst_stage_mask = stage;
      }
      BarrierType::General(__) => {
        __.dst_stage_mask = stage;
      }
      BarrierType::Buffer(__) => {
        __.dst_stage_mask = stage;
      }
    };
    *self
  }
  pub fn src_access(
    &mut self,
    access: vk::AccessFlags2,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.src_access_mask = access;
      }
      BarrierType::General(__) => {
        __.src_access_mask = access;
      }
      BarrierType::Buffer(__) => {
        __.src_access_mask = access;
      }
    };
    *self
  }
  pub fn dst_access(
    &mut self,
    access: vk::AccessFlags2,
  ) -> WBarr {
    match &mut self.barrier {
      BarrierType::Image(__) => {
        __.dst_access_mask = access;
      }
      BarrierType::General(__) => {
        __.dst_access_mask = access;
      }
      BarrierType::Buffer(__) => {
        __.dst_access_mask = access;
      }
    };
    *self
  }
  // fn image(&mut self, image: &WImage)->WBarr{
  // }
}
