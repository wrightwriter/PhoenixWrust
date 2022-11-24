use ash::vk;

use super::wdevice::WDevice;

pub struct WPipelineConfig{
  pub topology: vk::PrimitiveTopology,
  pub front_face: vk::FrontFace,
  pub blend_state: vk::PipelineColorBlendAttachmentState,
}

impl Default for WPipelineConfig{
  fn default() -> Self {
    Self { 
      topology: vk::PrimitiveTopology::TRIANGLE_LIST ,
      front_face: vk::FrontFace::CLOCKWISE,
      blend_state: vk::PipelineColorBlendAttachmentState::builder()
            .src_color_blend_factor(vk::BlendFactor::SRC_COLOR)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_DST_COLOR)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ZERO)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
              vk::ColorComponentFlags::R
                | vk::ColorComponentFlags::G
                | vk::ColorComponentFlags::B
                | vk::ColorComponentFlags::A,
            )
            .blend_enable(false)
            .build(),
    }
  }
}