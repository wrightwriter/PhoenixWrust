use std::collections::HashMap;

use ash::vk;

use crate::{
  res::{
    buff::{
      wpushconstant::WPushConstant, wuniformscontainer::UniformsContainer,
      wwritablebuffertrait::WWritableBufferTrait,
    },
    img::wrendertarget::{WRenderTarget, WRenderTargetInfo},
    wshader::WShaderEnumPipelineBind,
  },
  sys::{
    warenaitems::{
      WAIdxBindGroup, WAIdxRenderPipeline, WAIdxRt, WAIdxShaderProgram, WAIdxUbo, WArenaItem,
    },
    wdevice::GLOBALS,
    wrenderpipeline::{WRenderPipeline, WRenderPipelineTrait},
  },
  wvulkan::WVulkan,
};

pub trait WPassTrait {
  fn get_rt(&self) -> Option<WAIdxRt>;
  fn get_shader_program(&self) -> Option<WAIdxShaderProgram>;

  // pub rt: Option<WAIdxRt>,
}

pub struct WFxPass {
  pub rt: Option<WAIdxRt>,
  pub shader_program: WAIdxShaderProgram,
  pub render_pipeline: WAIdxRenderPipeline,

  pub ubo: WAIdxUbo,

  pub bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
  pub bind_group: WAIdxBindGroup,

  pub push_constants: UniformsContainer,
  pub uniforms: UniformsContainer,

  push_constants_internal: WPushConstant,
}
impl WFxPass {
  pub fn new(
    w_v: &mut WVulkan,
    has_rt: bool,
    shader_program: WAIdxShaderProgram,
  ) -> Self {
    let init_render_target = &mut w_v.w_swapchain.default_render_targets[0];
    let shared_bind_group = w_v.shared_bind_group;
    let mut render_pipeline = WAIdxRenderPipeline {
      idx: unsafe {
        (&mut *GLOBALS.shared_render_pipelines).insert(WRenderPipeline::new_passthrough_pipeline(
          &w_v.w_device.device,
        ))
      },
    };
    {
      render_pipeline.get_mut().init();
    }
    let rt;
    if has_rt {
      let rt_create_info = WRenderTargetInfo { ..wdef!() };
      rt = Some(
        w_v
          .w_tl
          .new_render_target(&mut w_v.w_device, rt_create_info)
          .0,
      );
    } else {
      rt = None;
    }
    let ubo = w_v.w_tl.new_uniform_buffer(&mut w_v.w_device, 1000).0;

    let mut personal_bind_group_idx = {
      let bind_group = w_v.w_grouper.new_group(&mut w_v.w_device);
      bind_group.1.set_binding_ubo(0, ubo.idx);

      // NEED TO REBUILD LATER TOO?
      bind_group.1.rebuild_all(
        &w_v.w_device.device,
        &w_v.w_device.descriptor_pool,
        &mut w_v.w_tl,
      );
      bind_group.0
    };

    let mut bind_groups = unsafe {
      let bind_groups = ptralloc!( HashMap<u32, WAIdxBindGroup>);
      std::ptr::write(bind_groups, HashMap::new());

      (*bind_groups).insert(0, shared_bind_group);
      (*bind_groups).insert(1, personal_bind_group_idx);

      bind_groups
    };

    unsafe {
      // let shader = &mut (*GLOBALS.shaders_arena)[prog_render.idx];
      match &mut (*GLOBALS.shader_programs_arena)[shader_program.idx].frag_shader {
        Some(__) => {
          __.pipelines
            .push(WShaderEnumPipelineBind::RenderPipeline(render_pipeline));
        }
        None => {}
      }
      match &mut (*GLOBALS.shader_programs_arena)[shader_program.idx].vert_shader {
        Some(__) => {
          __.pipelines
            .push(WShaderEnumPipelineBind::RenderPipeline(render_pipeline));
        }
        None => {}
      }
    }

    {
      render_pipeline
        .get_mut()
        .set_pipeline_bind_groups(&mut w_v.w_grouper, bind_groups);
    }
    {
      render_pipeline
        .get_mut()
        .set_pipeline_shader(shader_program);
    }
    {
      render_pipeline
        .get_mut()
        .set_pipeline_render_target(&init_render_target);
    }
    {
      render_pipeline.get_mut().refresh_pipeline(
        &w_v.w_device.device,
        &mut w_v.w_grouper,
        // bind_groups,
      );
    }

    Self {
      rt,
      shader_program,
      render_pipeline,
      push_constants: UniformsContainer::new(),
      uniforms: UniformsContainer::new(),
      push_constants_internal: WPushConstant::new(),
      ubo,
      bind_groups,
      bind_group: personal_bind_group_idx,
    }
  }

  pub fn run_on_internal_fb() {}

  pub fn run(
    &mut self,
    w_v: &mut WVulkan,
    command_buffer: &vk::CommandBuffer,
  ) {
    let w_device = &mut w_v.w_device;
    let w_grouper = &mut w_v.w_grouper;
    let w_tl = &mut w_v.w_tl;
    // -- UBO -- //

    let ubo = &mut self.ubo.get_mut().buff;
    ubo.reset_ptr();
    ubo.write_uniforms_container(&self.uniforms);

    unsafe {
      w_device
        .device
        .cmd_set_cull_mode(*command_buffer, vk::CullModeFlags::NONE);

      w_device
        .device
        .cmd_set_depth_test_enable(*command_buffer, false);
      w_device
        .device
        .cmd_set_depth_write_enable(*command_buffer, false);
      // w_device
      //   .device
      //   .cmd_set_front_face(*command_buffer, vk::FrontFace::);
    }

    // -- PUSH CONSTANTS -- //

    self.push_constants_internal.reset_ptr();

    let shared_ubo_bda_address = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena)[self.ubo.idx] // make this shorter? no?
      .buff
      .get_bda_address();

    self.push_constants_internal.write(shared_ubo_bda_address);

    self
      .push_constants_internal
      .write_uniforms_container(&self.push_constants);
    
    // -- BIND SHIT -- //

    unsafe{
      let mut sets: [vk::DescriptorSet; 2] = wmemzeroed!();
      for i in 0..2 {
        match (&*self.bind_groups).get(&i) {
          Some(__) => {
            sets[i as usize] = w_grouper.bind_groups_arena[__.idx].descriptor_set;
          }
          None => {}
        }
      }

      w_device.device.cmd_bind_descriptor_sets(
        *command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.render_pipeline.get_mut().pipeline_layout,
        0,
        &sets,
        &[],
      );
      w_device.device.cmd_bind_pipeline(
        *command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        self.render_pipeline.get_mut().pipeline,
      );

      w_device.device.cmd_push_constants(
        *command_buffer,
        self.render_pipeline.get_mut().pipeline_layout,
        vk::ShaderStageFlags::ALL,
        0,
        &self.push_constants_internal.array,
      );

      w_device.device.cmd_draw(*command_buffer, 4, 1, 0, 0);
    }
  }
}

impl WPassTrait for WFxPass {
  fn get_rt(&self) -> Option<WAIdxRt> {
    self.rt
  }
  fn get_shader_program(&self) -> Option<WAIdxShaderProgram> {
    Some(self.shader_program)
  }
}

impl WRenderPipelineTrait for WFxPass {
  fn get_pipeline(&self) -> &WRenderPipeline {
    &*self.render_pipeline.get_mut()
  }
}
