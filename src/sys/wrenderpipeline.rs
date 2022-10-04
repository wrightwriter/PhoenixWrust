use std::{
  borrow::BorrowMut,
  cell::Cell,
  ffi::CStr,
  ops::{ DerefMut}, collections::HashMap,
};

use ash::{
  vk,
};

use smallvec::SmallVec;

use crate::{
  wmemzeroed,
  res::wrendertarget::WRenderTarget,
  res::wshader::{WProgram}, sys::wmanagers::{ WGrouper, WAIdxBindGroup},
};

static entry_point: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

pub trait WRenderPipelineTrait {
  fn get_pipeline(&self) -> &WRenderPipeline;
}

pub struct WRenderPipeline {
  vertex_input: vk::PipelineVertexInputStateCreateInfo,

  input_assembly: vk::PipelineInputAssemblyStateCreateInfo,

  viewports: Vec<vk::Viewport>,

  scissors: Vec<vk::Rect2D>,
  viewport_state: vk::PipelineViewportStateCreateInfo,
  rasterizer: vk::PipelineRasterizationStateCreateInfo,

  multisampling: vk::PipelineMultisampleStateCreateInfo,

  color_blend_attachments: Vec<vk::PipelineColorBlendAttachmentState>,

  color_blending: vk::PipelineColorBlendStateCreateInfo,

  push_constant_range: vk::PushConstantRange,
  pipeline_layout_info: vk::PipelineLayoutCreateInfo,
  pub pipeline_layout: vk::PipelineLayout,

  // https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Conclusion
  rt_formats: Vec<vk::Format>,
  pipeline_rendering_info: vk::PipelineRenderingCreateInfo,

  pipeline_info: vk::GraphicsPipelineCreateInfo,

  pub pipeline: Cell<vk::Pipeline>,
  // pub set_layouts_vec: Vec<vk::DescriptorSetLayout>,
  pub set_layouts_vec: SmallVec<[vk::DescriptorSetLayout; 10]>,
}


impl WRenderPipeline {
  pub fn new_passthrough_pipeline(
    device: &ash::Device,
    // width: u32,
    // height: u32,
    // allocator: &mut GpuAllocator<vk::DeviceMemory>,
  ) -> Box<WRenderPipeline> {
    let a = SmallVec::<[u64;4]>::new();
    let mut w = Box::new(WRenderPipeline {
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

      // https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Conclusion
      rt_formats: wmemzeroed!(),
      pipeline_rendering_info: wmemzeroed!(),
      pipeline_info: wmemzeroed!(),
      pipeline: wmemzeroed!(),

      set_layouts_vec: wmemzeroed!(),
    });

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

    w.vertex_input = vk::PipelineVertexInputStateCreateInfo::builder()
      .deref_mut()
      .to_owned();

    w.input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
      .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
      .primitive_restart_enable(false)
      .deref_mut()
      .deref_mut()
      .to_owned();

    w.viewports = vec![vk::Viewport::builder()
      .x(0.0)
      .y(0.0)
      // .width(rend.resx as f32)
      // .height(default_render_targets[0].resy as f32)
      // .width(width as f32)
      // .height(height as f32)
      .min_depth(0.0)
      .max_depth(1.0)
      .build()
      ];

    w.scissors = vec![
      vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(extent)
        .build()
        , // .deref_mut().to_owned()
    ];
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

    w.color_blend_attachments = vec![
      vk::PipelineColorBlendAttachmentState::builder()
        .color_write_mask(
          vk::ColorComponentFlags::R
            | vk::ColorComponentFlags::G
            | vk::ColorComponentFlags::B
            | vk::ColorComponentFlags::A,
        )
        .blend_enable(false)
        .build()
        , // .deref_mut().to_owned()
    ];

    w.push_constant_range = vk::PushConstantRange::builder()
      .offset(0)
      .size(256)
      .stage_flags(vk::ShaderStageFlags::ALL)
      .build();

    w.pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
      .deref_mut()
      .to_owned();

    w.pipeline_layout_info.push_constant_range_count = 1;
    w.pipeline_layout_info.p_push_constant_ranges = &w.push_constant_range;

    // https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Conclusion

    // let rt_formats = &[default_render_targets[0].images()[0].format];

    w.rt_formats = vec![vk::Format::ASTC_5X5_SFLOAT_BLOCK];

    w.pipeline_rendering_info = vk::PipelineRenderingCreateInfo::builder()
      .deref_mut()
      .to_owned();
    // .color_attachment_formats(rt_formats);

    w.viewport_state = vk::PipelineViewportStateCreateInfo::builder()
      .viewports(&w.viewports)
      .scissors(&w.scissors)
      .deref_mut()
      .to_owned();

    w.color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
      .logic_op_enable(false)
      .attachments(&w.color_blend_attachments)
      .deref_mut()
      .to_owned();
    w.pipeline_layout =
      unsafe { device.create_pipeline_layout(&w.pipeline_layout_info, None) }.unwrap();

    w.pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
      .vertex_input_state(&w.vertex_input)
      .input_assembly_state(&w.input_assembly)
      .viewport_state(&w.viewport_state)
      .rasterization_state(&w.rasterizer)
      .multisample_state(&w.multisampling)
      .color_blend_state(&w.color_blending)
      .layout(w.pipeline_layout)
      // .render_pass(*default_render_targets.render_pass())
      .subpass(0)
      .build()
      ;
    w.pipeline_info.p_next = wtransmute!(&mut w.pipeline_rendering_info);



    // let pipeline = unsafe {
    //     device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
    // }.unwrap()[0];

    w.pipeline = wmemzeroed!();

    w
  }

  fn refresh_bind_group_layouts(
    &mut self,
    // bindings: &HashMap<u32, &dyn WTraitBinding>,
    w_grouper: &mut WGrouper,
    bind_groups: &HashMap<u32, WAIdxBindGroup>,
  ){
    self.set_layouts_vec.clear();

    // self.set_layouts_vec = bind_groups.iter().map(|binding|{
    //   let bind_group_layout = w_grouper.bind_groups_arena.get((*binding.1).idx).unwrap().descriptor_set_layout;
    //   bind_group_layout
    // }).collect();

    for i in 0..2 {
      match bind_groups.get(&i) {
          Some(__) => {
            let group = w_grouper.bind_groups_arena[__.idx].borrow_mut();
            // self.set_layouts_vec.push(bind_group_layout)
            let bind_group_layout = group.descriptor_set_layout;
            self.set_layouts_vec.push(bind_group_layout)
          },
          None => {},
      }
    }

    // bind_groups.iter().for_each(|binding| {
    //   let bind_group_layout = w_grouper.bind_groups_arena.get((*binding.1).idx).unwrap().descriptor_set_layout;
    //   self.set_layouts_vec.push(bind_group_layout)
    // });

    self.pipeline_layout_info.set_layout_count = self.set_layouts_vec.len() as u32;
    self.pipeline_layout_info.p_set_layouts = self.set_layouts_vec.as_ptr();
  }
  pub fn set_pipeline_bind_groups<'a>(
    &mut self,
    // bindings: &HashMap<u32, &dyn WTraitBinding>,
    w_grouper: &mut WGrouper,
    bind_groups: &HashMap<u32, WAIdxBindGroup>,
  ) {
    self.refresh_bind_group_layouts(w_grouper, bind_groups);
    // let mut bindings_vec = vec![];
  }

  pub fn refresh_pipeline(
    &mut self,
    device: &ash::Device,
    w_grouper: &mut WGrouper,
    bind_groups: &HashMap<u32, WAIdxBindGroup>,
  ) {
    self.refresh_bind_group_layouts(w_grouper, bind_groups);
    self.pipeline.set(
      unsafe {
        self.pipeline_layout =
          unsafe { device.create_pipeline_layout(&self.pipeline_layout_info, None) }.unwrap();
        // let info = std::mem::transmute(self.pipeline_info);
        // maybe not needed?
        self.pipeline_info.layout = self.pipeline_layout;
        let info = self.pipeline_info;
        device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)
      }
      .unwrap()[0],
    );
  }

  pub fn set_pipeline_render_target(
    &mut self,
    render_target: &WRenderTarget, // shader: crate::wshader::WProgram
  ) {

    unsafe {
      self.set_layouts_vec = SmallVec::new();
      let extent = vk::Extent2D {
        width: render_target.resx,
        height: render_target.resy,
      };

      self.viewports.clear();

      self.viewports.push(
        vk::Viewport::builder()
          .x(0.0)
          .y(0.0)
          .width(render_target.resx as f32)
          .height(render_target.resy as f32)
          .min_depth(0.0)
          .max_depth(1.0)
          .build()
          ,
      );

      self.viewport_state.p_viewports = std::mem::transmute(&self.viewports[0]);

      self.scissors.clear();
      self.scissors.push(
        vk::Rect2D::builder()
          .offset(vk::Offset2D { x: 0, y: 0 })
          .extent(extent)
          .build() 
          ,
      );
      // .deref_mut().to_owned()

      // (*self.viewport_state.p_scissors).extent = extent;

      // (*self.viewport_state.p_scissors).extenteViewportStateCreateInfo::builder()
      //     .viewports( &viewports)
      //     .scissors(&scissors).deref_mut().to_owned();

      // self.pipeline_info

      self.rt_formats.clear();
      self.rt_formats.push(render_target.images[0].format);

      // TODO: unneeded
      self.pipeline_rendering_info.p_color_attachment_formats =
        std::mem::transmute(&self.rt_formats[0]);
      // let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfo::builder().deref_mut().to_owned();
      //   .color_attachment_formats(rt_formats);

      // let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
      //     .vertex_input_state(&vertex_input)
      //     .input_assembly_state(&input_assembly)
      //     .viewport_state(&viewport_state)
      //     .rasterization_state(&rasterizer)
      //     .multisample_state(&multisampling)
      //     .color_blend_state(&color_blending)
      //     .layout(pipeline_layout)
      //     // .render_pass(*default_render_targets.render_pass())
      //     .subpass(0)
      //     .extend_from(&mut pipeline_rendering_info).deref_mut().to_owned();
    }
  }

  pub fn set_pipeline_shader(
    &mut self,
    shader: &WProgram,
  ) {
    unsafe {
      self.pipeline_info.p_stages = std::mem::transmute(&shader.stages[0]);
      self.pipeline_info.stage_count = 2;
    }
  }


}

// impl Default for WImage{
//     fn default() -> Self {
//         Self { handle: None, resx: 500, resy: 500, format: None, view: None }
//     }
// }
