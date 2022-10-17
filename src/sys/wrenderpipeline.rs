use std::{
  borrow::{Borrow, BorrowMut},
  cell::Cell,
  collections::HashMap,
  ffi::CStr,
  ops::DerefMut,
};

use ash::vk;

use smallvec::SmallVec;

use crate::{
  res::wrendertarget::WRenderTarget,
  res::wshader::WProgram,
  sys::wmanagers::{WGrouper},
  wmemzeroed,
};

use super::{
  wdevice::GLOBALS,
  warenaitems::{WAIdxShaderProgram, WArenaItem, WAIdxBindGroup},
};

static entry_point: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

pub trait WRenderPipelineTrait {
  fn get_pipeline(&self) -> &WRenderPipeline;
}

pub struct WRenderPipeline {
  pub vertex_input: vk::PipelineVertexInputStateCreateInfo,

  pub input_assembly: vk::PipelineInputAssemblyStateCreateInfo,

  pub viewports: *mut SmallVec<[vk::Viewport; 3]>,

  pub scissors: *mut SmallVec<[vk::Rect2D; 3]>,
  pub viewport_state: vk::PipelineViewportStateCreateInfo,
  pub rasterizer: vk::PipelineRasterizationStateCreateInfo,

  pub multisampling: vk::PipelineMultisampleStateCreateInfo,

  pub color_blend_attachments: *mut SmallVec<[vk::PipelineColorBlendAttachmentState; 3]>,

  pub color_blending: vk::PipelineColorBlendStateCreateInfo,

  pub push_constant_range: *mut vk::PushConstantRange,
  pub pipeline_layout_info: vk::PipelineLayoutCreateInfo,
  pub pipeline_layout: vk::PipelineLayout,

  // https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Conclusion
  pub rt_formats: *mut SmallVec<[vk::Format; 3]>,
  pub pipeline_rendering_info: vk::PipelineRenderingCreateInfo,

  pub pipeline_info: vk::GraphicsPipelineCreateInfo,

  pub dynamic_state_enables: SmallVec<[vk::DynamicState; 20]>,
  pub dynamic_state_info: vk::PipelineDynamicStateCreateInfo,

  pub shader_program: WAIdxShaderProgram,
  pub shader_stages: *mut SmallVec<[vk::PipelineShaderStageCreateInfo; 10]>,

  pub pipeline: vk::Pipeline,
  pub set_layouts_vec: *mut SmallVec<[vk::DescriptorSetLayout; 10]>,

  pub bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
}

impl WRenderPipeline {
  pub fn new_passthrough_pipeline(
    device: &ash::Device,
  ) -> WRenderPipeline {
    unsafe {
      let a = SmallVec::<[u64; 4]>::new();
      let mut w = WRenderPipeline {
        vertex_input: wmemzeroed!(),
        input_assembly: wmemzeroed!(),

        viewports: wmemzeroed!(),
        scissors: wmemzeroed!(),
        viewport_state: wmemzeroed!(),
        rasterizer: wmemzeroed!(),

        multisampling: wmemzeroed!(),

        color_blend_attachments: wmemzeroed!(),
        color_blending: wmemzeroed!(),

        push_constant_range: wmemzeroed!(),
        pipeline_layout_info: wmemzeroed!(),
        pipeline_layout: wmemzeroed!(),

        rt_formats: wmemzeroed!(),
        pipeline_rendering_info: wmemzeroed!(),
        pipeline_info: wmemzeroed!(),
        pipeline: wmemzeroed!(),

        bind_groups: wmemzeroed!(),

        set_layouts_vec: wmemzeroed!(),
        shader_program: wmemzeroed!(),
        shader_stages: wmemzeroed!(),

        dynamic_state_enables: wmemzeroed!(),
        dynamic_state_info: wmemzeroed!(),
      };


      let extent = vk::Extent2D {
        width: 100,
        height: 100,
      };

      // -- DYNAMIC STATE -- //

      // let dyn_viewports = [vk::Viewport::builder().width(40.).height(50.).x(0.).y(0.).build()];

      // let dyn_state = vk::DynamicState::VIEWPORT;

      // let dyn_viewport_state = vk::PipelineViewportStateCreateInfo::builder()
      //   .viewports(&dyn_viewports)
      //   .build();

      // let dynami_states = [
      //   vk::DynamicState::VIEWPORT
      // ];
      // let dynamic_state_create = vk::PipelineDynamicStateCreateInfo::builder()
      //   .dynamic_states(&dynami_states)
      // ;

      w.dynamic_state_enables = SmallVec::new();
      w.dynamic_state_enables.push(vk::DynamicState::CULL_MODE);
      // w.dynamic_state_enables.push(vk::DynamicState::VIEWPORT);
      // w.dynamic_state_enables.push(vk::DynamicState::SCISSOR);
      // w.dynamic_state_enables.push(vk::DynamicState::LINE_WIDTH);
      w.dynamic_state_enables.push(vk::DynamicState::DEPTH_TEST_ENABLE);
      // w.dynamic_state_enables.push(vk::DynamicState::DEPTH_COMPARE_OP);
      w.dynamic_state_enables.push(vk::DynamicState::DEPTH_WRITE_ENABLE);
      w.dynamic_state_enables.push(vk::DynamicState::FRONT_FACE);
      // w.dynamic_state_enables.push(vk::DynamicState::RASTERIZER_DISCARD_ENABLE);
      // w.dynamic_state_enables.push(vk::DynamicState::PRIMITIVE_TOPOLOGY);
      
      w.dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder().build();

      

      w.viewports = ptralloc!(SmallVec<[vk::Viewport; 3]>);
      std::ptr::write(w.viewports, SmallVec::new());

      w.set_layouts_vec = ptralloc!(SmallVec<[vk::DescriptorSetLayout; 10]>);
      std::ptr::write(w.set_layouts_vec, SmallVec::new());

      w.shader_stages = ptralloc!(SmallVec<[vk::PipelineShaderStageCreateInfo; 10]>);
      std::ptr::write(w.shader_stages, SmallVec::new());

      w.vertex_input = vk::PipelineVertexInputStateCreateInfo::builder()
        .deref_mut()
        .to_owned();

      w.input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .deref_mut()
        .deref_mut()
        .to_owned();

      w.viewports = ptralloc!(SmallVec<[vk::Viewport; 3]>);
      std::ptr::write(w.viewports, SmallVec::new());

      (*w.viewports).push(
        vk::Viewport::builder()
          .x(0.0)
          .y(0.0)
          // .width(rend.resx as f32)
          // .height(default_render_targets[0].resy as f32)
          // .width(width as f32)
          // .height(height as f32)
          .min_depth(0.0)
          .max_depth(1.0)
          .build(),
      );

      w.scissors = ptralloc!(SmallVec<[vk::Rect2D; 3]>);
      std::ptr::write(w.scissors, SmallVec::new());

      (*w.scissors).push(
        vk::Rect2D::builder()
          .offset(vk::Offset2D { x: 0, y: 0 })
          .extent(extent)
          .build(), // .deref_mut().to_owned()
      );
      // vec![
      // ];
      w.rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .line_width(1.0)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_clamp_enable(false)
        .deref_mut()
        .to_owned();

      w.multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
        .sample_shading_enable(false)
        .rasterization_samples(vk::SampleCountFlags::TYPE_1)
        .deref_mut()
        .to_owned();


      w.color_blend_attachments = ptralloc!(SmallVec<[vk::PipelineColorBlendAttachmentState; 3]>);
      std::ptr::write(w.color_blend_attachments, SmallVec::new());

      (*w.color_blend_attachments).push(
        vk::PipelineColorBlendAttachmentState::builder()
          .color_write_mask(
            vk::ColorComponentFlags::R
              | vk::ColorComponentFlags::G
              | vk::ColorComponentFlags::B
              | vk::ColorComponentFlags::A,
          )
          .blend_enable(false)
          .build(), // .deref_mut().to_owned()
      );

      // w.color_blend_attachments = vec![
      // ];

      w.push_constant_range = ptralloc!(vk::PushConstantRange);
      std::ptr::write(
        w.push_constant_range,
        vk::PushConstantRange::builder()
          .offset(0)
          .size(256)
          .stage_flags(vk::ShaderStageFlags::ALL)
          .build(),
      );

      w.pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
        .deref_mut()
        .to_owned();

      w.pipeline_layout_info.push_constant_range_count = 1;
      w.pipeline_layout_info.p_push_constant_ranges = w.push_constant_range;

      // https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Conclusion

      // let rt_formats = &[default_render_targets[0].images()[0].format];

      w.rt_formats = ptralloc!(SmallVec<[vk::Format; 3]>);
      std::ptr::write(w.rt_formats, SmallVec::new());

      (*w.rt_formats).push(vk::Format::ASTC_5X5_SFLOAT_BLOCK);
      // w.rt_formats = vec![];

      w.pipeline_rendering_info = vk::PipelineRenderingCreateInfo::builder()
        .deref_mut()
        .to_owned();
      // .color_attachment_formats(rt_formats);

      w.viewport_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(&*w.viewports)
        .scissors(&*w.scissors)
        .deref_mut()
        .to_owned();

      w.color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .attachments(&*w.color_blend_attachments)
        .deref_mut()
        .to_owned();

      w.pipeline_layout =
        unsafe { device.create_pipeline_layout(&w.pipeline_layout_info, None) }.unwrap();

      w.pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
        // .vertex_input_state(&w.vertex_input)
        // .input_assembly_state(&w.input_assembly)
        // .viewport_state(&w.viewport_state)
        // .rasterization_state(&w.rasterizer)
        // .multisample_state(&w.multisampling)
        // .color_blend_state(&w.color_blending)
        .layout(w.pipeline_layout)
        // .render_pass(*default_render_targets.render_pass())
        .subpass(0)
        .build();

      w.pipeline_layout_info.p_push_constant_ranges = w.push_constant_range;

      w.pipeline_info.p_next = wtransmute!(&mut w.pipeline_rendering_info);

      // let pipeline = unsafe {
      //     device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
      // }.unwrap()[0];

      w.pipeline = wmemzeroed!();

      w
    }
  }

  pub fn init(&mut self) {
    unsafe {
      self.viewport_state.p_viewports = (*self.viewports).as_ptr();
      // self.viewport_state.p_scissors = &(*self.scissors)[0];
      self.viewport_state.p_scissors = (*self.scissors).as_ptr();
      // self.color_blending.p_attachments = &(*self.color_blend_attachments)[0];
      self.color_blending.p_attachments = (*self.color_blend_attachments).as_ptr();

      self.pipeline_info.p_stages = (*self.shader_stages).as_ptr();
    }


    self.pipeline_info.p_vertex_input_state = &self.vertex_input;
    self.pipeline_info.p_input_assembly_state = &self.input_assembly;
    self.pipeline_info.p_viewport_state = &self.viewport_state;
    self.pipeline_info.p_rasterization_state = &self.rasterizer;
    self.pipeline_info.p_multisample_state = &self.multisampling;
    self.pipeline_info.p_color_blend_state = &self.color_blending;


    self.dynamic_state_info.p_dynamic_states = self.dynamic_state_enables.as_ptr();
    self.dynamic_state_info.dynamic_state_count = self.dynamic_state_enables.len() as u32;

    self.pipeline_info.p_dynamic_state = &self.dynamic_state_info;

    self.pipeline_layout_info.p_push_constant_ranges = self.push_constant_range;

    // self.pipeline_info.p_next = wtransmute!(&mut w.pipeline_rendering_info);
    // self.pipeline_info.p_next = wtransmute!(&self.pipeline_rendering_info);
    self.pipeline_info.p_next = wtransmute!(&self.pipeline_rendering_info);
  }

  fn refresh_bind_group_layouts(
    &mut self,
    // bindings: &HashMap<u32, &dyn WTraitBinding>,
    w_grouper: &WGrouper,
    bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
  ) {
    unsafe {
      (*self.set_layouts_vec).set_len(0);

      // self.set_layouts_vec = bind_groups.iter().map(|binding|{
      //   let bind_group_layout = w_grouper.bind_groups_arena.get((*binding.1).idx).unwrap().descriptor_set_layout;
      //   bind_group_layout
      // }).collect();
      let bind_groups = unsafe { &mut *bind_groups };
      for i in 0..2 {
        match bind_groups.get(&i) {
          Some(__) => {
            let group = w_grouper.bind_groups_arena[__.idx].borrow();
            // self.set_layouts_vec.push(bind_group_layout)
            let bind_group_layout = group.descriptor_set_layout;
            (*self.set_layouts_vec).push(bind_group_layout)
          }
          None => {}
        }
      }
    }

    // bind_groups.iter().for_each(|binding| {
    //   let bind_group_layout = w_grouper.bind_groups_arena.get((*binding.1).idx).unwrap().descriptor_set_layout;
    //   self.set_layouts_vec.push(bind_group_layout)
    // });

    unsafe {
      self.pipeline_layout_info.set_layout_count = (*self.set_layouts_vec).len() as u32;
      self.pipeline_layout_info.p_set_layouts = (*self.set_layouts_vec).as_ptr();
    }
  }
  pub fn set_pipeline_bind_groups(
    &mut self,
    // bindings: &HashMap<u32, &dyn WTraitBinding>,
    w_grouper: &mut WGrouper,
    bind_groups: *mut HashMap<u32, WAIdxBindGroup>,
  ) {
    self.bind_groups = bind_groups;

    self.refresh_bind_group_layouts(w_grouper, bind_groups);
    // let mut bindings_vec = vec![];
  }

  pub fn refresh_pipeline(
    &mut self,
    device: &ash::Device,
    w_grouper: &WGrouper,
    // bind_groups: &HashMap<u32, WAIdxBindGroup>,
  ) {
    self.refresh_bind_group_layouts(w_grouper, self.bind_groups);
    
    let mut pip = 
      unsafe {
        // let mut shader_stages: [vk::PipelineShaderStageCreateInfo; 2] = wmemzeroed!();
        (*self.shader_stages).set_len(0);
        for i in 0..2 {
          // (*self.shader_stages)[i] = (*GLOBALS.shaders_arena)[self.shader_program.idx].stages[i];
          (*self.shader_stages).push(
            (*GLOBALS.shader_programs_arena)[self.shader_program.idx].stages[i]
          );
        }


        self.pipeline_info.stage_count = 2;

        self.pipeline_layout =
          unsafe { device.create_pipeline_layout(&self.pipeline_layout_info, None) }.unwrap();

        self.pipeline_info.layout = self.pipeline_layout;

        let info = self.pipeline_info;
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)
      }
      .unwrap()[0];
    std::mem::swap(&mut pip, &mut self.pipeline);
    println!("refreshed pipelnie")
    // );
  }

  pub fn set_pipeline_render_target(
    &mut self,
    render_target: &WRenderTarget, // shader: crate::wshader::WProgram
  ) {
    unsafe {
      // self.set_layouts_vec = SmallVec::new();
      let extent = vk::Extent2D {
        width: render_target.resx,
        height: render_target.resy,
      };

      // (*self.viewports).clear();
      {
        (*self.viewports).set_len(0);
      }
      {
        (*self.viewports).push(
          vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(render_target.resx as f32)
            .height(render_target.resy as f32)
            .min_depth(0.0)
            .max_depth(1.0)
            .build(),
        );
      }

      (*self.scissors).set_len(0);
      (*self.scissors).push(
        vk::Rect2D::builder()
          .offset(vk::Offset2D { x: 0, y: 0 })
          .extent(extent)
          .build(),
      );

      (*self.rt_formats).set_len(0);
      (*self.rt_formats).push(render_target.images[0].format);

      // TODO: unneeded
      self.pipeline_rendering_info.p_color_attachment_formats =
        std::mem::transmute(&*self.rt_formats);
    }
  }

  pub fn set_pipeline_shader(
    &mut self,
    shader: WAIdxShaderProgram,
  ) {
    self.shader_program = shader
  }
}

// impl Default for WImage{
//     fn default() -> Self {
//         Self { handle: None, resx: 500, resy: 500, format: None, view: None }
//     }
// }
