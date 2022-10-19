use std::collections::HashMap;

use crate::{
  res::{
    wrendertarget::{WRenderTarget, WRenderTargetCreateInfo},
    wshader::WShaderEnumPipelineBind,
  },
  sys::{
    warenaitems::{WAIdxBindGroup, WAIdxRenderPipeline, WAIdxRt, WAIdxShaderProgram, WArenaItem},
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
}
impl WFxPass {
  pub fn new(
    w_v: &mut WVulkan,
    has_rt: bool,
    shared_bind_group: WAIdxBindGroup,
    shader_program: WAIdxShaderProgram,
    init_render_target: &WRenderTarget,
  ) -> Self {
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
      let rt_create_info = WRenderTargetCreateInfo { ..wdef!() };
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
    }
  }

  pub fn run_on_internal_fb() {}

  pub fn run() {}
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
