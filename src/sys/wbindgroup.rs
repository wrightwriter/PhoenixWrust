use std::borrow::BorrowMut;

use std::collections::HashMap;


use ash::vk;
use libc::c_void;

use crate::res::wbindings::{WBindingBufferArray, WBindingImageArray};
use crate::sys::warenaitems::{WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxUbo, WEnumBind};

use crate::sys::wmanagers::WTechLead;

use super::wdevice::{GLOBALS};

pub trait WBindGroupsHaverTrait {
  fn get_bind_groups(&self) -> &HashMap<u32, WAIdxBindGroup>;
}
pub struct WBindGroup {
  pub descriptor_set_layout: vk::DescriptorSetLayout,
  pub descriptor_set: vk::DescriptorSet,
  pub buffer_array_binding: Option<*mut WBindingBufferArray>,
  pub image_array_binding: Option<*mut WBindingImageArray>,
  pub bindings: HashMap<u32, WEnumBind>,
}

impl WBindGroup {
  pub fn set_image_array_binding(
    &mut self,
    index: u32,
    binding: &WBindingImageArray,
  ) {
  }

  pub fn set_binding_ubo(
    &mut self,
    index: u32,
    arena_idx: generational_arena::Index,
  ) {
    let bind = WEnumBind::WAIdxUbo(WAIdxUbo { idx: arena_idx });
    self.bindings.insert(index, bind);
  }

  pub fn set_binding_image(
    &mut self,
    index: u32,
    arena_idx: generational_arena::Index,
  ) {
    let bind = WEnumBind::WAIdxImage(WAIdxImage { idx: arena_idx });
    self.bindings.insert(index, bind);
  }

  pub fn set_binding_buffer(
    &mut self,
    index: u32,
    arena_idx: generational_arena::Index,
  ) {
    let bind = WEnumBind::WAIdxBuffer(WAIdxBuffer { idx: arena_idx });
    self.bindings.insert(index, bind);
  }

  pub fn new(
    device: &ash::Device,
    descriptor_pool: &vk::DescriptorPool,
  ) -> Self {
    Self {
      descriptor_set_layout: wmemzeroed!(),
      // descriptor_set_layout_bindings: wmemzeroed!(),
      descriptor_set: wmemzeroed!(),
      bindings: HashMap::new(),
      image_array_binding: None,
      buffer_array_binding: None,
    }
    // device.destroy_descriptor_set_layout(descriptor_set_layout, allocator)
    // vk::DescriptorPoolCreateFlags::
    // .p_bindings(&set_layout_binding);
  }
  pub fn rebuild_layout(
    &mut self,
    device: &ash::Device,
    descriptor_pool: &vk::DescriptorPool,
    w_tl: &mut WTechLead,
  ) {
    let mut vk_bindings: Vec<vk::DescriptorSetLayoutBinding> = vec![];

    // self.bindings.iter().map
    for binding in &self.bindings {
      let id = *binding.0;

      let bind_group_bind = binding.1;

      let set_layout_binding = {
        match bind_group_bind {
          WEnumBind::WAIdxImage(__) => vk::DescriptorSetLayoutBinding::builder()
            .binding(id)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .stage_flags(vk::ShaderStageFlags::ALL),
          WEnumBind::WAIdxUbo(__) => {
            vk::DescriptorSetLayoutBinding::builder()
              .binding(id)
              .descriptor_count(1)
              .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER) // TODO: dynamic
              .stage_flags(vk::ShaderStageFlags::ALL)
          }
          WEnumBind::WAIdxBuffer(__) => {
            vk::DescriptorSetLayoutBinding::builder()
              .binding(id)
              .descriptor_count(1)
              .descriptor_type(vk::DescriptorType::STORAGE_BUFFER) // TODO: dynamic
              .stage_flags(vk::ShaderStageFlags::ALL)
          }
        }
      };
      vk_bindings.push(set_layout_binding.build());
    }

    if let Some(img_array_binding) = self.image_array_binding {
      unsafe {
        let cnt = (*img_array_binding).count;
        vk_bindings.push(
          vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_count(cnt)
            .stage_flags(vk::ShaderStageFlags::ALL)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .build(),
        );
        vk_bindings.push(
          vk::DescriptorSetLayoutBinding::builder()
            .binding(2)
            .descriptor_count(cnt)
            .stage_flags(vk::ShaderStageFlags::ALL)
            .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
            .build(),
        );
        vk_bindings.push(
          vk::DescriptorSetLayoutBinding::builder()
            .binding(3)
            .descriptor_count(3)
            .stage_flags(vk::ShaderStageFlags::ALL)
            .descriptor_type(vk::DescriptorType::SAMPLER)
            .build(),
        );
        vk_bindings.push(
          vk::DescriptorSetLayoutBinding::builder()
            .binding(4)
            .descriptor_count(cnt)
            .stage_flags(vk::ShaderStageFlags::ALL)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .build(),
        );
      }
    }

    // if let Some(img_array_binding) = self.image_array_binding {
    //   // let img_array_binding = img_array_binding.borrow();

    //   unsafe {
    //     let write = vk::WriteDescriptorSet::builder()
    //         .dst_binding(1)
    //         .dst_array_element(0)
    //         .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
    //         .dst_set(self.descriptor_set)
    //         .image_info(&(*img_array_binding).vk_infos)
    //         .build();

    //     device.update_descriptor_sets(&[write], &[]);
    //   }
    // }

    let mut layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
      .bindings(&vk_bindings)
      .build();
    

    // (0..layout_info.binding_count).map(||{vk::DescriptorBindingFlags::PARTIALLY_BOUND});
    let binding_frags = vec![vk::DescriptorBindingFlags::PARTIALLY_BOUND; layout_info.binding_count as usize];
    // let 
    let binding_flags_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
      .binding_flags(&
          binding_frags
        // (0..layout_info.binding_count).map(||{vk::DescriptorBindingFlags::PARTIALLY_BOUND})
        // vk::DescriptorBindingFlags::PARTIALLY_BOUND 
        )
      .build();
    // binding_flags.binding_count = layout_info.binding_count;


    layout_info.p_next = ((&binding_flags_info) as *const vk::DescriptorSetLayoutBindingFlagsCreateInfo) as *const c_void;
    // layout_info.p_next = &binding_flags as *const c_void;
    self.descriptor_set_layout = unsafe {
      device
        .create_descriptor_set_layout(&layout_info, None)
        .unwrap()
    };

    // allocate single descriptor set
    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
      .descriptor_pool(*descriptor_pool)
      .set_layouts(&[self.descriptor_set_layout])
      .build();

    self.descriptor_set = unsafe {
      device
        .allocate_descriptor_sets(&descriptor_set_allocate_info)
        .unwrap()
    }[0];
  }

  pub fn rebuild_descriptors(
    &mut self,
    device: &ash::Device,
    descriptor_pool: &vk::DescriptorPool,
    w_tl: &mut WTechLead,
  ) {
    // ! Write descriptor set 📰
    {
      let mut writes: Vec<vk::WriteDescriptorSet> = vec![];

      for binding in &self.bindings {
        let id = *binding.0;
        let bind_group_bind = binding.1;


        let set_write = {
          match bind_group_bind {
              WEnumBind::WAIdxImage(__) => {
                let img = w_ptr_to_mut_ref!(GLOBALS.shared_images_arena)[__.idx].borrow_mut();
                let img_info = vk::DescriptorImageInfo::builder()
                  .image_view(img.view)
                  // .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                  // .image_layout(vk::ImageLayout::GENERAL)
                  .image_layout(img.descriptor_image_info.image_layout)
                  .build();

                writes.push(
                  vk::WriteDescriptorSet::builder()
                    .dst_binding(id)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
                    .dst_set(self.descriptor_set)
                    .image_info(&[img_info])
                    .build(),
                )
              }
              WEnumBind::WAIdxUbo(__) => {
                let ubo = w_ptr_to_mut_ref!( GLOBALS.shared_ubo_arena )[__.idx].borrow_mut();
                let ubo_info = vk::DescriptorBufferInfo::builder()
                  .buffer(ubo.buff.get_handle())
                  .offset(0)
                  .range(ubo.buff.sz_bytes.into())
                  .build();

                writes.push(
                  vk::WriteDescriptorSet::builder()
                    .dst_binding(id)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .dst_set(self.descriptor_set)
                    .buffer_info(&[ubo_info])
                    .build(),
                )
                // vk::DescriptorSetLayoutBinding::builder()
                //   .binding(id)
                //   .descriptor_count(1)
                //   .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER) // TODO: dynamic
                //   .stage_flags(vk::ShaderStageFlags::ALL)
              }
              WEnumBind::WAIdxBuffer(__) => {
                // let buff = w_tl.shared_buffers_arena[__.idx].borrow_mut();

                let buff = unsafe {
                  (*GLOBALS.shared_buffers_arena)[__.idx].borrow_mut()
                };

                let buff_info = vk::DescriptorBufferInfo::builder()
                  .buffer(buff.get_handle())
                  .offset(0)
                  .range(buff.sz_bytes.into())
                  .build();

                writes.push(
                  vk::WriteDescriptorSet::builder()
                    .dst_binding(id)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .dst_set(self.descriptor_set)
                    .buffer_info(&[buff_info])
                    .build(),
                )
                // vk::DescriptorSetLayoutBinding::builder()
                //   .binding(id)
                //   .descriptor_count(1)
                //   .descriptor_type(vk::DescriptorType::STORAGE_BUFFER) // TODO: dynamic
                //   .stage_flags(vk::ShaderStageFlags::ALL)
              }
              // WBindGroupBind::WBindTypeImageArray(__) => {
              //   // let v = value;
              //   // let a = *__;
              //   let img_array = __;
              //   vk::DescriptorSetLayoutBinding::builder()
              //     .binding(id)
              //     .descriptor_count(img_array.count)
              //     .stage_flags(vk::ShaderStageFlags::ALL)
              //     .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
              // },
            }
        };
      }

      unsafe {
        device.update_descriptor_sets(&writes, &[]);
      }
    }
    if let Some(img_array_binding) = self.image_array_binding {
      // let img_array_binding = img_array_binding.borrow();

      unsafe {
        let mut writes: [vk::WriteDescriptorSet; 4] = wmemzeroed!();
        
// TODO: epic lazy static? 🔥
        let linear_sampler_info = vk::SamplerCreateInfo::builder()
          .mag_filter(vk::Filter::LINEAR)
          .min_filter(vk::Filter::LINEAR)
          .address_mode_u(vk::SamplerAddressMode::REPEAT)
          .address_mode_v(vk::SamplerAddressMode::REPEAT)
          .build();
        let linear_sampler = device.create_sampler(&linear_sampler_info, None).unwrap();
        
        let mipmap_sampler_info = vk::SamplerCreateInfo::builder()
          .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
          .min_lod(0.0)
          .max_lod(9.0)
          .mip_lod_bias(0.)
          .mag_filter(vk::Filter::LINEAR)
          .min_filter(vk::Filter::LINEAR)
          .address_mode_u(vk::SamplerAddressMode::REPEAT)
          .address_mode_v(vk::SamplerAddressMode::REPEAT)
          .build();
        let mipmap_sampler = device.create_sampler(&mipmap_sampler_info, None).unwrap();
        
        
        let sampler_infos = [
          vk::DescriptorImageInfo::builder().sampler(linear_sampler).build(),
          vk::DescriptorImageInfo::builder().sampler(mipmap_sampler).build(),
          vk::DescriptorImageInfo::builder().sampler(linear_sampler).build(),
        ];
        

        writes[0] = vk::WriteDescriptorSet::builder()
          .dst_binding(1)
          .dst_array_element(0)
          .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
          .dst_set(self.descriptor_set)
          .image_info(&(*img_array_binding).vk_infos_storage)
          .build();
        writes[1] = vk::WriteDescriptorSet::builder()
          .dst_binding(2)
          .dst_array_element(0)
          .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
          .dst_set(self.descriptor_set)
          .image_info(&(*img_array_binding).vk_infos_sampled)
          .build();
        writes[2] = vk::WriteDescriptorSet::builder()
          .dst_binding(3)
          .dst_array_element(0)
          .descriptor_type(vk::DescriptorType::SAMPLER)
          .dst_set(self.descriptor_set)
          .image_info(&sampler_infos)
          .build();

        // let last_write = ;

        writes[3] = vk::WriteDescriptorSet::builder()
          .dst_binding(4)
          .dst_array_element(0)
          .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
          .dst_set(self.descriptor_set)
          // .image_info(&sampler_infos)
          .buffer_info(&(*self.buffer_array_binding.unwrap()).vk_infos)
          .build();

        // writes[3] = vk::WriteDescriptorSet::builder()
        //   .dst_binding(4)
        //   .dst_array_element(0)
        //   .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
        //   .dst_set(self.descriptor_set)
        //   // .image_info(&sampler_infos)
        //   .buffer_info(&(*self.buffer_array_binding.unwrap()).vk_infos)
        //   .build();

        device.update_descriptor_sets(&writes, &[]);
      }
    }
  }

  pub fn rebuild_all(
    &mut self,
    device: &ash::Device,
    descriptor_pool: &vk::DescriptorPool,
    w_tl: &mut WTechLead,
  ) {
    self.rebuild_layout(device, descriptor_pool, w_tl);
    self.rebuild_descriptors(device, descriptor_pool, w_tl);
  }
}
