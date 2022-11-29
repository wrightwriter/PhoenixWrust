use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::DerefMut;

use ash::vk;
use ash::vk::BufferCollectionConstraintsInfoFUCHSIA;
use bytemuck::Contiguous;
use nalgebra_glm::Mat4;
use nalgebra_glm::Vec3;
use nalgebra_glm::Vec4;
use smallvec::SmallVec;

use crate::res::buff::wpushconstant::WPushConstant;
use crate::res::buff::wuniformscontainer::WParamsContainer;
use crate::res::buff::wwritablebuffertrait::WWritableBufferTrait;
use crate::res::img::wrendertarget::WRenderTarget;
use crate::res::wmodel::WModel;
use crate::res::wshader::WProgram;
use crate::res::wshader::WShaderEnumPipelineBind;
use crate::sys::warenaitems::WAIdxBindGroup;
use crate::sys::warenaitems::WAIdxRenderPipeline;
use crate::sys::warenaitems::WAIdxRt;
use crate::sys::warenaitems::WAIdxShaderProgram;
use crate::sys::warenaitems::WAIdxUbo;
use crate::sys::warenaitems::WArenaItem;
use crate::sys::wdevice::WDevice;
use crate::sys::wdevice::GLOBALS;
use crate::sys::wrenderpipeline::WRenderPipeline;
use crate::sys::wrenderstate::WRenderState;
use crate::sys::wtl::WTechLead;
use crate::wvulkan::WVulkan;

pub trait WThingTrait {
  fn get_rt(&self) -> Option<WAIdxRt>;
  fn get_push_constants_internal(&mut self) -> &mut WPushConstant;
  fn get_ubo(&mut self) -> &mut WAIdxUbo;
  fn get_uniforms_container(&mut self) -> &mut WParamsContainer;
  fn get_push_constants(&mut self) -> &mut WParamsContainer;
  fn get_bind_groups(&mut self) -> *mut HashMap<u32, WAIdxBindGroup>;
  fn get_pipeline(&mut self) -> &mut WAIdxRenderPipeline;
  fn get_render_state(&mut self) -> &mut WRenderState;

  fn init_render_settings(
    &mut self,
    w_device: &mut WDevice,
    w_tl: &mut WTechLead,
    cmd_buf: &vk::CommandBuffer,
  ) {
    let bind_groups = self.get_bind_groups();
    let render_pipeline = *self.get_pipeline();

    unsafe {
      // -- DYNAMIC STATE -- //
      self.get_render_state().run(*cmd_buf, w_device);

      let mut sets: [vk::DescriptorSet; 2] = wmemzeroed!();
      for i in 0..2 {
        match (&*bind_groups).get(&i) {
          Some(__) => {
            sets[i as usize] = (&*GLOBALS.bind_groups_arena)[__.idx].descriptor_set;
          }
          None => {}
        }
      }

      self.get_render_state().run(*cmd_buf, w_device);

      w_device.device.cmd_bind_descriptor_sets(
        *cmd_buf,
        vk::PipelineBindPoint::GRAPHICS,
        render_pipeline.get().pipeline_layout,
        0,
        &sets,
        &[],
      );

      w_device
        .device
        .cmd_bind_pipeline(*cmd_buf, vk::PipelineBindPoint::GRAPHICS, render_pipeline.get().pipeline);
    }
  }

  // pub fn update_push_constants(&mut self, w_device: &WDevice, command_buffer: &vk::CommandBuffer){
  fn upload_push_constants(
    &mut self,
    // push_constants_internal: &mut WPushConstant,
    w_device: &WDevice,
    command_buffer: &vk::CommandBuffer,
  ) {
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

    self.upload_push_constants(w_device, command_buffer);
  }

  fn run(
    &mut self,
    w_v: &mut WVulkan,
    w_tl: &mut WTechLead,
    command_buffer: &vk::CommandBuffer,
  ) {
    let w_device = &mut w_v.w_device;
    // let w_grouper = &mut w_v.w_grouper;

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
}

#[macro_export]
macro_rules! declare_thing {
    ($struct:ident {$( $field:ident:$type:ty ),*}) =>{
    pub struct $struct {
        $(
            pub $field: $type,
        )*

        pub rt: Option<WAIdxRt>,
        // pub shader_program: WAIdxShaderProgram,
        pub render_pipeline: WAIdxRenderPipeline,

        pub ubo: WAIdxUbo,

        pub render_state: WRenderState,

        pub bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
        pub bind_group: WAIdxBindGroup,

        pub push_constants: WParamsContainer,
        // pub uniforms: WUniformsContainer,

        pub movable: bool,
        pub world_pos: Vec3,
        pub model_mat: Mat4,

        push_constants_internal: WPushConstant,
    }
  }
}

#[macro_export]
macro_rules! impl_thing_trait {
  ($struct:ident {$( $field:ident:$type:ty ),*}) => {
    impl WThingTrait for $struct {
      fn get_rt(&self) -> Option<WAIdxRt> {
        self.rt
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

      fn get_push_constants(&mut self) -> &mut WParamsContainer {
        &mut self.push_constants
      }

      fn get_render_state(&mut self) -> &mut WRenderState {
        &mut self.render_state
      }

      fn get_bind_groups(&mut self) -> *mut HashMap<u32, WAIdxBindGroup> {
        self.bind_groups
      }

      fn get_pipeline(&mut self) -> &mut WAIdxRenderPipeline {
        &mut self.render_pipeline
      }
    }

    impl WBindGroupsHaverTrait for $struct {
      fn get_bind_groups(&self) -> &HashMap<u32, WAIdxBindGroup> {
        unsafe { &*self.bind_groups }
      }
    }
    impl WRenderPipelineTrait for $struct {
      fn get_pipeline(&self) -> &WRenderPipeline {
        &*self.render_pipeline.get_mut()
      }
    }
  };
}

pub fn init_thing_stuff(
  w_v: &mut WVulkan,
  w_tl: &mut WTechLead,
  prog_render: WAIdxShaderProgram,
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
  WRenderState,
) {
  let mut render_pipeline = WAIdxRenderPipeline {
    idx: unsafe { (&mut *GLOBALS.shared_render_pipelines).insert(WRenderPipeline::new_passthrough_pipeline(&w_v.w_device.device)) },
  };
  {
    let rp = render_pipeline.get_mut();
    rp.input_assembly.topology = vk::PrimitiveTopology::TRIANGLE_LIST;
    rp.init();
  }

  let ubo = w_tl.new_uniform_buffer(&mut w_v.w_device, 1000).0;

  let mut personal_bind_group_idx = unsafe {
    let bind_group_idx = w_tl.new_group(&mut w_v.w_device).0;
    let bind_group = &mut (*GLOBALS.bind_groups_arena)[bind_group_idx.idx];
    bind_group.set_binding_ubo(0, ubo.idx);

    bind_group.rebuild_all(&w_v.w_device.device, &w_v.w_device.descriptor_pool, w_tl);
    bind_group_idx
  };

  let mut bind_groups = unsafe {
    let bind_groups = ptralloc!( HashMap<u32, WAIdxBindGroup>);
    std::ptr::write(bind_groups, HashMap::new());

    (*bind_groups).insert(0, w_v.shared_bind_group);
    (*bind_groups).insert(1, personal_bind_group_idx);

    bind_groups
  };

  unsafe {
    // let shader = &mut (*GLOBALS.shaders_arena)[prog_render.idx];
    match &mut (*GLOBALS.shader_programs_arena)[prog_render.idx].frag_shader {
      Some(__) => {
        __.pipelines.push(WShaderEnumPipelineBind::RenderPipeline(render_pipeline));
      }
      None => {}
    }
    match &mut (*GLOBALS.shader_programs_arena)[prog_render.idx].vert_shader {
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
    render_pipeline.get_mut().set_pipeline_shader(prog_render);
  }
  {
    let init_render_target = &mut w_v.w_swapchain.default_render_targets[0];
    render_pipeline.get_mut().set_pipeline_render_target(&init_render_target);
  }
  {
    render_pipeline.get_mut().refresh_pipeline(
      &w_v.w_device.device,
      &w_tl,
      // bind_groups,
    );
  }

  (
    None,
    prog_render,
    render_pipeline,
    ubo,
    bind_groups,
    personal_bind_group_idx,
    WParamsContainer::new(),
    WParamsContainer::new(),
    WPushConstant::new(),
    WRenderState::default(),
  )
}
