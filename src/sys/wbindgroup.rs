use std::any::Any;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use std::{collections::HashMap, mem::MaybeUninit};

use ash::vk;

use getset::Getters;
use gpu_alloc::GpuAllocator;
use gpu_alloc::MemoryDevice;
use spirv_reflect::ffi::SpvAddressingModel__SpvAddressingModelLogical;

use crate::res::wbindings::{WBindingAttachmentTrait, WBindingBufferArray, WBindingImageArray};
// use crate::wbuffer::WBuffer;
// use crate::wbuffer::WBuffer;
use crate::sys::wmanagers::{
  WAIdxBindGroup, WAIdxBuffer, WAIdxImage, WAIdxUbo, WBinding, WBindingAttachmentEnum, WEnumBind,
  WTechLead,
};

// #[derive(PartialEq, Eq, Hash)]
pub trait WBindGroupsHaverTrait {
  // fn get_bind_set(&self) -> &WBindSet;
  fn get_bind_groups(&self) -> &HashMap<u32, WAIdxBindGroup>;
}
pub struct WBindGroup {
  pub descriptor_set_layout: vk::DescriptorSetLayout,
  // pub descriptor_set_layout_bindings: Vec<vk::DescriptorSetLayoutBinding>,
  pub descriptor_set: vk::DescriptorSet,
  // pub bindings: Vec<vk::DescriptorSetLayoutBindingBuilder<'a>>,
  // pub bindings: HashMap<u32, &dyn WBindingAttachmentTrait>,
  pub buffer_array_binding: Option<Rc<RefCell<WBindingBufferArray>>>,
  pub image_array_binding: Option<Rc<RefCell<WBindingImageArray>>>,
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
    
    if let Some(img_array_binding) = &self.image_array_binding {
      let img_array_binding = img_array_binding.borrow();
      vk_bindings.push(
        vk::DescriptorSetLayoutBinding::builder()
          .binding(1)
          .descriptor_count(img_array_binding.count)
          .stage_flags(vk::ShaderStageFlags::ALL)
          .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
          .build()
      )
    }


    let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
      .bindings(&vk_bindings)
      ;
    self.descriptor_set_layout = unsafe {
      device
        .create_descriptor_set_layout(&layout_info, None)
        .unwrap()
    };

    // allocate single descriptor set
    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
      .descriptor_pool(*descriptor_pool)
      .set_layouts(&[self.descriptor_set_layout])
      .build()
      ;

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
                let img = w_tl.shared_images_arena[__.idx].borrow_mut();
                let img_info = vk::DescriptorImageInfo::builder()
                  .image_view(*img.view())
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
                let ubo = w_tl.shared_ubo_arena[__.idx].borrow_mut();
                let ubo_info = vk::DescriptorBufferInfo::builder()
                  .buffer(ubo.buff.handle)
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
                let buff = w_tl.shared_buffers_arena[__.idx].borrow_mut();
                let buff_info = vk::DescriptorBufferInfo::builder()
                  .buffer(buff.handle)
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
    if let Some(img_array_binding) = &mut self.image_array_binding {
      let img_array_binding = img_array_binding.borrow();

      let write = vk::WriteDescriptorSet::builder()
          .dst_binding(1)
          .dst_array_element(0)
          .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
          .dst_set(self.descriptor_set)
          .image_info(&img_array_binding.vk_infos)
          .build();

      unsafe {
        device.update_descriptor_sets(&[write], &[]);
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