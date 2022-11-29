use std::collections::HashMap;

use ash::vk;
use macros::add_uniform;
use macros::init_uniform;

use crate::sys::wtl::WTechLead;
use crate::{
  res::{
    buff::{wpushconstant::WPushConstant, wuniformscontainer::WParamsContainer, wwritablebuffertrait::WWritableBufferTrait},
    img::wrendertarget::WRTInfo,
    wshader::WShaderEnumPipelineBind,
  },
  sys::{
    warenaitems::{WAIdxBindGroup, WAIdxRenderPipeline, WAIdxRt, WAIdxShaderProgram, WAIdxUbo, WArenaItem},
    wdevice::{WDevice, GLOBALS},
    pipeline::wrenderpipeline::{WRenderPipeline, WRenderPipelineTrait},
  },
  wvulkan::WVulkan,
};

pub fn init_fx_pass_stuff(
  w_v: &mut WVulkan,
  w_tl: &mut WTechLead,
  has_rt: bool,
  shader_program: WAIdxShaderProgram,
) -> (
  Option<WAIdxRt>,
  WAIdxShaderProgram,
  WAIdxRenderPipeline,
  WAIdxUbo,
  *mut HashMap<u32, WAIdxBindGroup>,
  WAIdxBindGroup,
  WParamsContainer,
  WParamsContainer,
  WPushConstant,
) {
  let rt;
  if has_rt {
    let rt_create_info = WRTInfo { ..wdef!() };
    rt = Some(w_tl.new_render_target(w_v, rt_create_info).0);
  } else {
    rt = None;
  }

  // let w_tl = w_t_l;
  let init_render_target = &mut w_v.w_swapchain.default_render_targets[0];
  let shared_bind_group = w_v.shared_bind_group;
  let mut render_pipeline = WAIdxRenderPipeline {
    idx: unsafe { (&mut *GLOBALS.shared_render_pipelines).insert(WRenderPipeline::new_passthrough_pipeline(&w_v.w_device.device)) },
  };
  {
    let rp = render_pipeline.get_mut();
    rp.input_assembly.topology = vk::PrimitiveTopology::TRIANGLE_STRIP;
    rp.init();
  }
  let ubo = w_tl.new_uniform_buffer(&mut w_v.w_device, 1000).0;

  let mut personal_bind_group_idx = unsafe {
    let bind_group_idx = w_tl.new_group(&mut w_v.w_device).0;
    let bind_group = &mut (*GLOBALS.bind_groups_arena)[bind_group_idx.idx];
    bind_group.set_binding_ubo(0, ubo.idx);

    // NEED TO REBUILD LATER TOO?
    bind_group.rebuild_all(&w_v.w_device.device, &w_v.w_device.descriptor_pool, w_tl);
    bind_group_idx
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
        __.pipelines.push(WShaderEnumPipelineBind::RenderPipeline(render_pipeline));
      }
      None => {}
    }
    match &mut (*GLOBALS.shader_programs_arena)[shader_program.idx].vert_shader {
      Some(__) => {
        __.pipelines.push(WShaderEnumPipelineBind::RenderPipeline(render_pipeline));
      }
      None => {}
    }
  }

  {
    render_pipeline.get_mut().set_pipeline_bind_groups(w_tl, bind_groups);
  }
  {
    render_pipeline.get_mut().set_pipeline_shader(shader_program);
  }
  {
    render_pipeline.get_mut().set_pipeline_render_target(&init_render_target);
  }
  {
    render_pipeline.get_mut().refresh_pipeline(
      &w_v.w_device.device,
      w_tl,
      // bind_groups,
    );
  };

  (
    rt,
    shader_program,
    render_pipeline,
    ubo,
    bind_groups,
    personal_bind_group_idx,
    WParamsContainer::new(),
    WParamsContainer::new(),
    WPushConstant::new(),
  )
}

pub trait WPassTrait {
  fn get_rt(&self) -> Option<WAIdxRt>;
  fn set_rt(
    &mut self,
    rt: WAIdxRt,
  );
  fn get_shader_program(&self) -> Option<WAIdxShaderProgram>;

  fn get_push_constants_internal(&mut self) -> &mut WPushConstant;
  fn get_ubo(&mut self) -> &mut WAIdxUbo;
  fn get_uniforms_container(&mut self) -> &mut WParamsContainer;
  fn get_push_constants(&mut self) -> &mut WParamsContainer;
  fn get_bind_groups(&mut self) -> *mut HashMap<u32, WAIdxBindGroup>;
  fn get_pipeline(&mut self) -> &mut WAIdxRenderPipeline;

  fn init_render_settings(
    &mut self,
    w_device: &mut WDevice,
    w_tl: &mut WTechLead,
    command_buffer: &vk::CommandBuffer,
  ) {
    let bind_groups = self.get_bind_groups();
    let render_pipeline = *self.get_pipeline();

    unsafe {
      w_device.device.cmd_set_cull_mode(*command_buffer, vk::CullModeFlags::NONE);

      w_device.device.cmd_set_depth_test_enable(*command_buffer, false);
      w_device.device.cmd_set_depth_write_enable(*command_buffer, false);

      let mut sets: [vk::DescriptorSet; 2] = wmemzeroed!();
      for i in 0..2 {
        match (&*bind_groups).get(&i) {
          Some(__) => {
            sets[i as usize] = (&*GLOBALS.bind_groups_arena)[__.idx].descriptor_set;
          }
          None => {}
        }
      }

      w_device.device.cmd_bind_descriptor_sets(
        *command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        render_pipeline.get_mut().pipeline_layout,
        0,
        &sets,
        &[],
      );

      w_device
        .device
        .cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, render_pipeline.get_mut().pipeline);
    }
  }

  // pub fn update_push_constants(&mut self, w_device: &WDevice, command_buffer: &vk::CommandBuffer){
  fn update_push_constants(
    &mut self,
    // push_constants_internal: &mut WPushConstant,
    w_device: &WDevice,
    command_buffer: &vk::CommandBuffer,
  ) {
    let ubo = *self.get_ubo();

    let shared_ubo_bda_address = unsafe {
      (*GLOBALS.shared_ubo_arena)[ubo.idx] // make this shorter? no?
        .buff
        .get_bda_address()
    };

    {
      let pc = self.get_push_constants().clone();

      let push_constants_internal = self.get_push_constants_internal();
      push_constants_internal.reset_ptr();

      push_constants_internal.write(shared_ubo_bda_address);

      push_constants_internal.write_params_container(&pc);
    }

    unsafe {
      w_device.device.cmd_push_constants(
        *command_buffer,
        self.get_pipeline().get().pipeline_layout,
        // render_pipeline.get_mut().pipeline_layout,
        vk::ShaderStageFlags::ALL,
        0,
        &self.get_push_constants_internal().array,
      );
    }
  }

  fn run(
    &mut self,
    w_v: &mut WVulkan,
    w_t_l: &mut WTechLead,
    // W_T_L
    command_buffer: &vk::CommandBuffer,
  ) {
    let w_device = &mut w_v.w_device;
    // let w_grouper = &mut w_v.w_grouper;
    let w_tl = w_t_l;

    WParamsContainer::reset_ptr(*self.get_ubo());
    WParamsContainer::upload_uniforms(*self.get_ubo(), &self.get_uniforms_container());
    self.init_render_settings(w_device, w_tl, command_buffer);
    self.update_push_constants(w_device, command_buffer);

    let ubo = *self.get_ubo();
    let ubo = &mut ubo.get_mut().buff;

    let uniforms_container = self.get_uniforms_container();

    ubo.reset_ptr();
    ubo.write_params_container(&uniforms_container);

    unsafe {
      w_device.device.cmd_draw(*command_buffer, 4, 1, 0, 0);
    }
  }

  fn run_on_internal_rt(
    &mut self,
    w_v: &mut WVulkan,
    w_t_l: &mut WTechLead,
    command_buffer: &vk::CommandBuffer,
  ) -> vk::CommandBuffer {
    let rt = self.get_rt().unwrap();
    let rt = rt.get_mut();
    rt.begin_pass(&mut w_v.w_device);

    self.run(w_v, w_t_l, &rt.cmd_buf);

    rt.end_pass(&mut w_v.w_device);
    rt.cmd_buf
  }

  fn run_on_external_rt(
    &mut self,
    rt_idx: WAIdxRt,
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
  ) -> vk::CommandBuffer {
    let rt = rt_idx.get_mut();
    rt.begin_pass(&mut w_v.w_device);

    // TODO: CODE DUPLICATION
    if self.get_rt().is_none() {
      self.set_rt(rt_idx);
      let rp = self.get_pipeline().get_mut();
      rp.set_pipeline_render_target(rt);
      rp.refresh_pipeline(&w_v.w_device.device, &w_tl);
    }

    self.run(w_v, w_tl, &rt.cmd_buf);

    rt.end_pass(&mut w_v.w_device);
    rt.cmd_buf
  }

  // pub rt: Option<WAIdxRt>,
}

#[macro_export]
macro_rules! declare_pass {
    ($struct:ident {$( $field:ident:$type:ty ),*}) =>{
        pub struct $struct {
            $(
                $field: $type,
            )*

            pub rt: Option<WAIdxRt>,
            pub shader_program: WAIdxShaderProgram,
            pub render_pipeline: WAIdxRenderPipeline,

            pub ubo: WAIdxUbo,

            pub bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
            pub bind_group: WAIdxBindGroup,

            pub push_constants: WParamsContainer,
            // pub uniforms: WUniformsContainer,

            push_constants_internal: WPushConstant,
        }

        // impl Trait for $struct {
        //     pub fn access_var(&mut self, var: bool) {
        //         self.var = var;
        //     }
        // }

      impl WPassTrait for $struct {
        fn get_rt(&self) -> Option<WAIdxRt> {
          self.rt
        }
        fn set_rt(&mut self, rt: WAIdxRt) {
          self.rt = Some(rt);
        }
        fn get_shader_program(&self) -> Option<WAIdxShaderProgram> {
          Some(self.shader_program)
        }

        fn get_push_constants_internal(&mut self) -> &mut WPushConstant {
          &mut self.push_constants_internal
        }
        fn get_ubo(&mut self) -> &mut WAIdxUbo {
          &mut self.ubo
        }

        fn get_uniforms_container(&mut self) -> &mut WParamsContainer {
          &mut self.get_ubo().get_mut().uniforms
        }

        fn get_push_constants(&mut self) -> &mut WParamsContainer{
          &mut self.push_constants
        }

        fn get_bind_groups(&mut self) -> *mut HashMap<u32, WAIdxBindGroup> {
          self.bind_groups
        }

        fn get_pipeline(&mut self) -> &mut WAIdxRenderPipeline {
          &mut self.render_pipeline
        }
      }
    };
}

// #[macro_export]
// macro_rules! impl_pass {
//     ($struct:ident {$( $field:ident:$type:ty ),*}) =>{

//     };
// }

