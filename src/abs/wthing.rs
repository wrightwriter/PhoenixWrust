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

use crate::res::buff::wpushconstant::WPushConstant;
use crate::res::buff::wwritablebuffertrait::WWritableBufferTrait;
use crate::res::wmodel::WModel;
use crate::res::img::wrendertarget::WRenderTarget;
use crate::res::wshader::WProgram;
use crate::res::wshader::WShaderEnumPipelineBind;
use crate::res::buff::wuniformscontainer::UniformsContainer;
use crate::sys::warenaitems::WAIdxBindGroup;
use crate::sys::warenaitems::WAIdxRenderPipeline;
use crate::sys::warenaitems::WAIdxRt;
use crate::sys::warenaitems::WAIdxShaderProgram;
use crate::sys::warenaitems::WAIdxUbo;
use crate::sys::warenaitems::WArenaItem;
use crate::sys::wbindgroup::WBindGroupsHaverTrait;
use crate::sys::wdevice::WDevice;
use crate::sys::wdevice::GLOBALS;
use crate::sys::wmanagers::WGrouper;
use crate::sys::wmanagers::WTechLead;
use crate::sys::wrenderpipeline::WRenderPipeline;
use crate::sys::wrenderpipeline::WRenderPipelineTrait;
use crate::wvulkan::WVulkan;

pub struct WThing {
  pub render_pipeline: WAIdxRenderPipeline,
  pub bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
  pub bind_group: WAIdxBindGroup,

  pub ubo: WAIdxUbo,
  
  pub rt: Option<WAIdxRt>,

  pub push_constants: UniformsContainer,
  pub uniforms: UniformsContainer,

  push_constants_internal: WPushConstant,

  pub model: Option<WModel>,
  pub movable: bool,
  pub world_pos: Vec3,
  pub model_mat: Mat4,
}

impl WThing {
  pub fn new(
    w_v: &mut WVulkan,
    prog_render: WAIdxShaderProgram,
  ) -> Self {
    let w_device = &mut w_v.w_device;
    let groupder = &mut w_v.w_grouper;
    let w_tech_lead = &mut w_v.w_tl;
    let shared_bind_group = w_v.shared_bind_group;
    let init_render_target = &mut w_v.w_swapchain.default_render_targets[0];

    let mut render_pipeline = WAIdxRenderPipeline {
      idx: unsafe {
        (&mut *GLOBALS.shared_render_pipelines)
          .insert(WRenderPipeline::new_passthrough_pipeline(&w_device.device))
      },
    };
    {
      let rp = render_pipeline.get_mut();
      rp.input_assembly.topology = vk::PrimitiveTopology::TRIANGLE_LIST;
      rp.init();
    }

    let ubo = w_tech_lead.new_uniform_buffer(w_device, 1000).0;

    let mut personal_bind_group_idx = {
      let bind_group = groupder.new_group(w_device);
      bind_group.1.set_binding_ubo(0, ubo.idx);

      // NEED TO REBUILD LATER TOO?
      bind_group
        .1
        .rebuild_all(&w_device.device, &w_device.descriptor_pool, w_tech_lead);
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
      render_pipeline
        .get_mut()
        .set_pipeline_bind_groups(groupder, bind_groups);
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
      movable: false,
      world_pos: Vec3::zeros(),
      model_mat: Mat4::identity(),
      model: None,
      rt: None,
      push_constants: UniformsContainer::new(),
      uniforms: UniformsContainer::new(),
      push_constants_internal: WPushConstant::new(),
    }
  }

  pub fn draw(
    &mut self,
    // w_device: &mut WDevice,
    // w_grouper: &mut WGrouper,
    // w_tl: &WTechLead,
    w_v: &mut WVulkan,
    rt: Option<WAIdxRt>,
    command_buffer: &vk::CommandBuffer,
  ) {
    let w_device = &mut w_v.w_device;
    let w_grouper = &mut w_v.w_grouper;
    let w_tl = &mut w_v.w_tl;
    if let Some(rt) = rt {
      if self.rt.is_none(){
        self.rt = Some(rt);

        let rp = self.render_pipeline.get_mut();
        rp.set_pipeline_render_target(rt.get_mut());
        rp.refresh_pipeline(&w_device.device, w_grouper);
      }
    }
    let ubo = &mut self.ubo.get_mut().buff; 
    ubo.reset_ptr();
    ubo.write_mat4x4(self.model_mat); 
    ubo.write_uniforms_container(&self.uniforms);

    
    
    unsafe {

      // let viewports = (*self.render_pipeline.get_mut().viewports.;
        
      // w_device.device.cmd_set_viewport(
      //   *command_buffer,
      //   0,
      //   viewports
      // );
      // w_device.device.cmd_set_scissor(*command_buffer, vk::CullModeFlags::BACK);
      // w_device.device.cmd_set_line_width(*command_buffer, 0.2f32);
      // w_device.device.cmd_eq(*command_buffer, vk::FrontFace::CLOCKWISE);
      // w_device.device.cmd_set_rasterizer_discard_enable(*command_buffer, vk::CullModeFlags::BACK);

      // -- DYNAMIC STATE -- //
      

      w_device
        .device
        // .cmd_set_cull_mode(*command_buffer, vk::CullModeFlags::BACK);
        .cmd_set_cull_mode(*command_buffer, vk::CullModeFlags::NONE);

      w_device.device.cmd_set_depth_test_enable(*command_buffer, true);
      w_device.device.cmd_set_depth_write_enable(*command_buffer, true);



      w_device.device.cmd_set_front_face(*command_buffer, vk::FrontFace::COUNTER_CLOCKWISE);
    }

    unsafe {
      // -- BIND SHIT -- //
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
      
      


      // -- PUSH CONSTANTS -- //
      self.push_constants_internal.reset_ptr();

      let shared_ubo_bda_address = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena)[self.ubo.idx] // make this shorter? no?
        .buff
        .get_bda_address();

      self.push_constants_internal.write(shared_ubo_bda_address);

      // -- DRAW -- //
      if let Some(model) = &self.model {
        for mesh in &model.meshes{
          self.push_constants_internal.reset_ptr();

          let shared_ubo_bda_address = w_ptr_to_mut_ref!(GLOBALS.shared_ubo_arena)[self.ubo.idx] // make this shorter? no?
            .buff
            .get_bda_address();

          let indices_arena_idx = mesh.gpu_indices_buff.get_mut().arena_index;
          let verts_arena_idx = mesh.gpu_verts_buff.get_mut().arena_index;

          self.push_constants_internal.write(shared_ubo_bda_address);

          self.push_constants_internal.write(indices_arena_idx.idx.index as u8 - 1);
          self.push_constants_internal.write(verts_arena_idx.idx.index as u8 - 1);

          self.push_constants_internal.write(model.textures[0].idx.index as u8 + mesh.material.diffuse_tex_idx as u8);
          self.push_constants_internal.write(model.textures[1].idx.index as u8 + mesh.material.normal_tex_idx as u8);
          self.push_constants_internal.write(model.textures[2].idx.index as u8 + mesh.material.metallic_roughness_tex_idx as u8);
          self.push_constants_internal.write(model.textures[3].idx.index as u8 + mesh.material.occlusion_tex_idx as u8);
          // self.push_constants_internal.write(node.mesh.material.normal_tex_idx as u8 - 1);
          // self.push_constants_internal.write(node.mesh.material.metallic_roughness_tex_idx as u8 - 1);
          // self.push_constants_internal.write(node.mesh.material.occlusion_tex_idx as u8 - 1);

          // self.push_constants_internal.write(node.mesh.material.diffuse_tex_idx as u8 - 1);
          // self.push_constants_internal.write(node.mesh.material.normal_tex_idx as u8 - 1);
          // self.push_constants_internal.write(node.mesh.material.metallic_roughness_tex_idx as u8 - 1);
          // self.push_constants_internal.write(node.mesh.material.occlusion_tex_idx as u8 - 1);


          self.push_constants_internal.write_uniforms_container(&self.push_constants);

          w_device.device.cmd_push_constants(
            *command_buffer,
            self.render_pipeline.get_mut().pipeline_layout,
            vk::ShaderStageFlags::ALL,
            0,
            &self.push_constants_internal.array,
          );
          
          w_device
            .device
            .cmd_draw(*command_buffer, mesh.indices_len, 1, 0, 0);
        }



      } else {

        self.push_constants_internal.write_uniforms_container(&self.push_constants);
        w_device.device.cmd_push_constants(
          *command_buffer,
          self.render_pipeline.get_mut().pipeline_layout,
          vk::ShaderStageFlags::ALL,
          0,
          &self.push_constants_internal.array,
        );
        w_device.device.cmd_draw(*command_buffer, 3, 1, 0, 0);
      }
      // *((ptr as *mut i32).offset(2) as *mut i32) = w.frame as i32;
      // w_device.device.cmd_push_constants(
      //   *command_buffer,
      //   self.render_pipeline.get_mut().pipeline_layout,
      //   vk::ShaderStageFlags::ALL,
      //   0,
      //   &push_constant.array,
      // );
      // w_device.device.cmd_draw(*command_buffer, 3, 1, 0, 0);
    }
  }
}
impl WBindGroupsHaverTrait for WThing {
  fn get_bind_groups(&self) -> &HashMap<u32, WAIdxBindGroup> {
    unsafe { &*self.bind_groups }
  }
}
impl WRenderPipelineTrait for WThing {
  fn get_pipeline(&self) -> &WRenderPipeline {
    &*self.render_pipeline.get_mut()
  }
}
