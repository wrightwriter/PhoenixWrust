use std::collections::HashMap;

use ash::vk;

use ash::vk::CommandBuffer;

use crate::res::wshader::WProgram;
use crate::sys::wbindgroup::WBindGroupsHaverTrait;
use crate::sys::wcomputepipeline::WComputePipeline;
use crate::sys::wdevice::WDevice;
use crate::sys::wmanagers::WAIdxBindGroup;
use crate::sys::wmanagers::WGrouper;
use crate::sys::wmanagers::WTechLead;

// !! ---------- IMAGE ---------- //

// mod crate::wbuffer;
// use WBindingContainer;
// mod wab
// use WAbstra

// #[derive(Getters)]

pub struct WComputePass<'a> {
  pub compute_pipeline: WComputePipeline,
  pub shader_program: &'a WProgram,
  pub command_buffer: CommandBuffer,
  pub bind_groups: HashMap<u32, WAIdxBindGroup>,
}

impl WComputePass<'_> {
  pub fn new(
    w_device: &mut WDevice,
    w_grouper: &mut WGrouper,
    w_tech_lead: &mut WTechLead,
    shared_bind_group: WAIdxBindGroup,
    shader_program: &WProgram,
  ) -> Self {
    let mut compute_pipeline = WComputePipeline::new(&w_device.device, shader_program);

    let ubo = w_tech_lead.new_uniform_buffer(w_device, 1000).0;

    // let mut bind_group = Box::new(WBindGroup::new(device, descriptor_pool));

    let mut personal_bind_group_idx = {
      let bind_group = w_grouper.new_group(w_device);
      bind_group.1.set_binding_ubo(0, ubo.idx);

      // NEED TO REBUILD LATER TOO
      bind_group
        .1
        .rebuild_all(&w_device.device, &w_device.descriptor_pool, w_tech_lead);
      bind_group.0
    };

    let mut bind_groups = HashMap::new();

    bind_groups.insert(0, shared_bind_group);
    bind_groups.insert(1, personal_bind_group_idx);

    compute_pipeline.set_pipeline_bind_groups(w_grouper, &bind_groups);

    compute_pipeline.refresh_pipeline(
      &w_device.device,
      w_grouper,
      &bind_groups,
    );


    let command_buffer = unsafe {
      let cmd_buf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(*&w_device.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        // .command_buffer_count(default_render_target.framebuffers().len() as _);
        .command_buffer_count(1);
      w_device
        .device
        .allocate_command_buffers(&cmd_buf_allocate_info).unwrap()[0]
    };

    unsafe {
      let sp = wptr!(shader_program, WProgram);

      Self {
        compute_pipeline,
        shader_program: sp,
        command_buffer,
        bind_groups,
      }
    }
  }

  pub fn dispatch(
    &self,
    w_device: &WDevice,
    w_grouper: &WGrouper,
    wkg_sz_x: u32,
    wkg_sz_y: u32,
    wkg_sz_z: u32,
  ) {
    let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder();
    unsafe {
      // let barrier = vk::MemoryBarri
      w_device.device.reset_command_buffer(
        self.command_buffer,
        vk::CommandBufferResetFlags::RELEASE_RESOURCES,
      );
      w_device.device
        .begin_command_buffer(self.command_buffer, &cmd_buf_begin_info)
        .unwrap();


      let mut sets = vec![];
      for i in (0..2){
        match self.bind_groups.get(&i) {
            Some(__) => {
              sets.push(w_grouper.bind_groups_arena[__.idx].descriptor_set)
            },
            None => {},
        }
      }

      w_device.device.cmd_bind_descriptor_sets(
        self.command_buffer,
        vk::PipelineBindPoint::COMPUTE,
        self.compute_pipeline.pipeline_layout,
        0,
        &sets,
        &[],
      );
      w_device.device.cmd_bind_pipeline(
        self.command_buffer,
        vk::PipelineBindPoint::COMPUTE,
        self.compute_pipeline.pipeline.get()
      );


      w_device.device.cmd_dispatch(self.command_buffer, wkg_sz_x, wkg_sz_y, wkg_sz_z);
      

      w_device.device.end_command_buffer(self.command_buffer).unwrap();
    }
  }
}

impl WBindGroupsHaverTrait for WComputePass<'_> {
  fn get_bind_groups(&self) -> &HashMap<u32, WAIdxBindGroup> {
    &self.bind_groups
  }
}

// impl Default for WImage{
//     fn default() -> Self {
//         Self { handle: None, resx: 500, resy: 500, format: None, view: None }
//     }
// }
