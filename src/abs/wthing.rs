use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::hash::Hash;

use ash::vk;


use crate::res::wrendertarget::WRenderTarget;
use crate::res::wshader::WShaderEnumPipelineBind;
use crate::sys::wbindgroup::WBindGroupsHaverTrait;
use crate::sys::wdevice::GLOBALS;
use crate::sys::wdevice::WDevice;
use crate::sys::wmanagers::WAIdxBindGroup;
use crate::sys::wmanagers::WAIdxRenderPipeline;
use crate::sys::wmanagers::WAIdxShaderProgram;
use crate::sys::wmanagers::WAIdxUbo;
use crate::sys::wmanagers::WArenaItem;
use crate::sys::wmanagers::WGrouper;
use crate::sys::wmanagers::WTechLead;
use crate::sys::wrenderpipeline::WRenderPipeline;
use crate::sys::wrenderpipeline::WRenderPipelineTrait;
use crate::res::wshader::WProgram;

pub struct WThing {
  pub render_pipeline: WAIdxRenderPipeline,
  pub bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
  pub bind_group: WAIdxBindGroup,
  pub ubo: WAIdxUbo,
}

impl WThing {
  pub fn new(
    w_device: &mut WDevice,
    groupder: &mut WGrouper,
    w_tech_lead: &mut WTechLead,
    shared_bind_group: WAIdxBindGroup,
    init_render_target: &WRenderTarget,
    prog_render: WAIdxShaderProgram,
  ) -> Self {
    let mut render_pipeline = WAIdxRenderPipeline {
      idx: unsafe {
        (&mut *GLOBALS.shared_render_pipelines)
          .insert(
            WRenderPipeline::new_passthrough_pipeline(&w_device.device)
          )
      },
    };
    {
      render_pipeline.get_mut().init();
    }


    let ubo = w_tech_lead.new_uniform_buffer(w_device, 1000).0;


    let mut personal_bind_group_idx = {
      let bind_group = groupder.new_group(w_device);
      bind_group.1.set_binding_ubo(0, ubo.idx);
      

      // NEED TO REBUILD LATER TOO?
      bind_group.1.rebuild_all( &w_device.device, &w_device.descriptor_pool, w_tech_lead);
      bind_group.0
    };


    let mut bind_groups = unsafe{
      let bind_groups = ptralloc!( HashMap<u32, WAIdxBindGroup>);
      std::ptr::write(bind_groups, HashMap::new());

      (*bind_groups).insert(0, shared_bind_group);
      (*bind_groups).insert(1, personal_bind_group_idx);

      bind_groups
    };

    unsafe {
      // let shader = &mut (*GLOBALS.shaders_arena)[prog_render.idx];
      match &mut (*GLOBALS.shader_programs_arena)[prog_render.idx].frag_shader {
        Some(__) => {
          __.pipelines
            .push(WShaderEnumPipelineBind::RenderPipeline(render_pipeline));
        }
        None => {}
      }
      match &mut (*GLOBALS.shader_programs_arena)[prog_render.idx].vert_shader {
        Some(__) => {
          __.pipelines
            .push(WShaderEnumPipelineBind::RenderPipeline(render_pipeline));
        }
        None => {}
      }
    }


    {
        render_pipeline.get_mut().set_pipeline_bind_groups(groupder, bind_groups);
    }
    {
        render_pipeline.get_mut().set_pipeline_shader(prog_render);
    }
    {
      render_pipeline
        .get_mut()
        .set_pipeline_render_target(&init_render_target);
    }
    {
      render_pipeline.get_mut().refresh_pipeline(
        &w_device.device,
        groupder,
        // bind_groups,
      );
    }


    

    Self {
      render_pipeline,
      // render_pipeline_box: render_pipeline_box,
      bind_groups,
      bind_group: personal_bind_group_idx,
      ubo,
    }
  }

  pub fn draw(
    &self,
    w_device: &mut WDevice,
    w_grouper: &mut WGrouper,
    w_tl: &WTechLead,
    command_buffer: &vk::CommandBuffer,
  ) {
    unsafe {
        // init thing
        let push_constant: [u8; 256] = wmemzeroed!();
        let mut ptr = push_constant.as_ptr();

        let shared_ubo_bda_address = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena)[self.ubo.idx] // make this shorter? no?
          .buff
          .get_bda_address();

        *((ptr as *mut i32).offset(0) as *mut u64) = shared_ubo_bda_address;
        // *((ptr as *mut i32).offset(2) as *mut i32) = w.frame as i32;

        w_device.device.cmd_push_constants(
          *command_buffer,
          self.render_pipeline.get_mut().pipeline_layout,
          vk::ShaderStageFlags::ALL,
          0,
          &push_constant,
        );

      let mut sets : [vk::DescriptorSet; 2] = wmemzeroed!();
      for i in 0..2 {
        match (&*self.bind_groups).get(&i) {
            Some(__) => {
              // sets.push(w_grouper.bind_groups_arena[__.idx].descriptor_set)
              sets[i as usize] = w_grouper.bind_groups_arena[__.idx].descriptor_set;
            },
            None => {},
        }
      }

      w_device.device.cmd_bind_descriptor_sets(
        *command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.render_pipeline.get_mut().pipeline_layout,
        0,
        // &sets[0],
        // self.bind_groups.len(),
        &sets,
        &[],
        // dynamic_offsets,
      );
      w_device.device.cmd_bind_pipeline(
        *command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.render_pipeline.get_mut().pipeline,
      );
      w_device.device.cmd_draw(*command_buffer, 3, 1, 0, 0);
    }
  }
}
impl WBindGroupsHaverTrait for WThing {
  fn get_bind_groups(&self) -> &HashMap<u32, WAIdxBindGroup> {
    unsafe{
      &*self.bind_groups
    }
  }
}
impl WRenderPipelineTrait for WThing {
  fn get_pipeline(&self) -> &WRenderPipeline {
    &*self.render_pipeline.get_mut()
  }
}
