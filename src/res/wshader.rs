use std::cell::Cell;
use std::fs;
use std::mem::MaybeUninit;

use ash::vk::ShaderModule;

// use egui::TextBuffer;
use shaderc::{self, ShaderKind};
use shaderc::{IncludeCallbackResult, IncludeType, ResolvedInclude};

use ash::vk;

use ash::Device;

use smallvec::SmallVec;
use tracy_client::span;

use std::ffi::CStr;

use crate::sys::warenaitems::WAIdxRenderPipeline;
use crate::sys::warenaitems::{WAIdxComputePipeline, WAIdxShaderProgram};
use crate::sys::wdevice::GLOBALS;

static entry_point: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

// !! ---------- IMAGE ---------- //

enum ProgramType {
  Render,
  Compute,
}
#[derive(Clone, Copy)]
pub enum WShaderEnumPipelineBind {
  ComputePipeline(WAIdxComputePipeline),
  RenderPipeline(WAIdxRenderPipeline),
}

pub struct WShader {
  pub kind: ShaderKind,
  pub file_name: String,
  shader_folder: String,
  pub txt: String,
  pub compilation_error: String,
  pub stage: Cell<vk::PipelineShaderStageCreateInfo>,
  pub module: Cell<ShaderModule>,
  pub pipelines: SmallVec<[WShaderEnumPipelineBind; 32]>, // pub binary: *mut u8
}

impl WShader {
  fn new(
    device: &Device,
    kind: ShaderKind,
    folder: String,
    file_name: String,
  ) -> Self {
    let mut s = Self {
      kind: kind.clone(),
      file_name: file_name.clone(),
      shader_folder: folder,
      txt: unsafe { MaybeUninit::zeroed().assume_init() },
      stage: unsafe { MaybeUninit::zeroed().assume_init() },
      module: unsafe { MaybeUninit::zeroed().assume_init() },
      compilation_error: String::from(""),
      pipelines: SmallVec::new(),
    };
    s.try_compile(device);
    s
  }
  #[profiling::function]
  pub fn try_compile(
    &mut self,
    device: &Device,
  ) {
    let mut txt: String = unsafe { MaybeUninit::zeroed().assume_init() };

    let full_path = &(self.shader_folder.clone() + &self.file_name);

    let include_folder = self.shader_folder.clone() + "includes\\";

    match fs::read_to_string(full_path) {
      Ok(v) => {
        txt = v;
        Result::Ok(String::from(""))
        // txt= v.parse().unwrap()
      }
      Err(e) => {
        debug_assert!(false);
        Result::Err(())
      }
    };

    let compiler = unsafe { &mut (*GLOBALS.compiler) };

    let include_callback =
      |include_file_name: &str, include_type: IncludeType, includer_name: &str, include_depth: usize| -> IncludeCallbackResult {
        // match include_type {
        //   IncludeType::Relative => todo!(),
        //   IncludeType::Standard => todo!(),
        // };

        let include_path = include_folder.clone() + &include_file_name;
        let include_text = fs::read_to_string(&include_path).unwrap();

        IncludeCallbackResult::Ok(ResolvedInclude {
          resolved_name: include_path,
          content: include_text,
        })
      };

    let mut options = shaderc::CompileOptions::new().unwrap();
    // shaderc::CompileOptions::set_generate_debug_info(&mut options);
    shaderc::CompileOptions::set_target_spirv(&mut options, shaderc::SpirvVersion::V1_4);
    shaderc::CompileOptions::add_macro_definition(&mut options, "scalar-block-layout", None);
    shaderc::CompileOptions::add_macro_definition(&mut options, "disable-spirv-val", None);
    shaderc::CompileOptions::set_target_env(&mut options, shaderc::TargetEnv::Vulkan, shaderc::EnvVersion::Vulkan1_2 as u32);

    shaderc::CompileOptions::set_include_callback(
      &mut options,
      include_callback, // F: Fn(&str, IncludeType, &str, usize) -> IncludeCallbackResult + 'a,
    );

    let shared_import_string_glsl = "#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_buffer_reference_uvec2 : require
#extension GL_EXT_nonuniform_qualifier  : require 
#extension GL_EXT_scalar_block_layout : enable
#extension GL_EXT_shader_8bit_storage : enable
#extension GL_EXT_shader_16bit_storage : enable
#extension GL_EXT_shader_image_load_formatted : require
#extension GL_EXT_shader_atomic_float: require
// #extension GLSL_EXT_debug_printf : require
    

#include \"global.include\"
      ";
    let wip = "
// These define pointer types.
// layout(buffer_reference, std430, buffer_reference_align = 16) readonly buffer ReadVec4
layout(buffer_reference, scalar, buffer_reference_align = 1, align = 1) readonly buffer ReadVec4 {
    vec4 values[];
};
// layout(buffer_reference, std430, buffer_reference_align = 16) writeonly buffer WriteVec4
// {
//     vec4 values[];
// };

// layout(buffer_reference, std430, buffer_reference_align = 4) readonly buffer UnalignedVec4
// {
//     vec4 value;
// };
    ";
    let shared_import_string_lower = "
// layout(set = 0, binding=0, std430) uniform SharedUbo{
layout(set = 0, binding=0, scalar, buffer_reference_align = 1, align = 1) uniform SharedUbo{
  vec3 camPos;



  vec2 R;

  vec2 mousePos;
  vec2 deltaMousePos;

  float T;
  float dT;
  uint frame;
  float RMBDown;
  float LMBDown;
  float zNear;
  float zFar;



  mat4 V;
  mat4 P;
  mat4 PV;
  mat4 invV;
  mat4 invP;

  mat4 Vprev;
  mat4 Pprev;
  mat4 PVprev;
  mat4 invVprev;
  mat4 invPprev;
}; 
layout(set = 0, binding = 1) uniform image2D shared_images[];
layout(set = 0, binding = 2) uniform texture2D shared_textures[];
layout(set = 0, binding = 3) uniform sampler shared_samplers[];
layout(set = 0, binding = 4) uniform sampler shared_ubos[];
// layout(set = 0, binding = 4) uniform sampler shared_buffers[];


      ";

    let push_constant_string_upper = "layout( push_constant, std430 ) uniform constants{
    ";
    let push_constant_content = "
      UboObject ubo;

    ";

    let push_constant_string_lower = "} PC;
    ";

    let dont_preprocess_regex = regex::Regex::new(r"\#W_DONT_PREPROCESS").unwrap();
    if let Some(__) = dont_preprocess_regex.find(&txt) {
      txt = dont_preprocess_regex.replace(&txt, "").to_string();
      txt = txt + shared_import_string_glsl;
    } else {
      let mut import_txt = "".to_string();

      let mut shared_import_string = shared_import_string_glsl.to_string();

      // skip if not found

      let regex_bda = regex::Regex::new(r"(?ms)W_BDA_DEF(.*?)\{(.*?)\}").unwrap();
      txt = regex_bda
        .replace_all(
          &txt,
          "layout(buffer_reference, scalar, buffer_reference_align = 1, align = 1) buffer $1 { $2 }",
        )
        .to_string();

      // -- PC DIRECTIVE
      let regex_pc = regex::Regex::new(r"(?ms)W_PC_DEF[ ]*\{(.*?)\}").unwrap();
      let regex_ubo = regex::Regex::new(r"(?ms)W_UBO_DEF[ ]*\{(.*?)\}").unwrap();

      let mut txt_clone = txt.clone();

      let mut regex_pc_found = regex_pc.find(&txt_clone);
      let mut regex_ubo_found = regex_ubo.find(&txt_clone);

      if regex_pc_found.is_some() && regex_ubo_found.is_none() {
        txt = regex::Regex::new(r"(?ms)W_PC_DEF[ ]*\{(.*?)\}")
          .unwrap()
          .replace(
            &txt,
            "
W_UBO_DEF{ float amoge; }
W_PC_DEF{ $1 }",
          )
          .to_string();
      } else if regex_pc_found.is_none() && regex_ubo_found.is_some() {
        txt = regex::Regex::new(r"(?ms)W_UBO_DEF[ ]*\{(.*?)\}")
          .unwrap()
          .replace(
            &txt,
            "
W_UBO_DEF{ $1}
W_PC_DEF{ 
 UboObject ubo;         
}
             ",
          )
          .to_string();
      } else if regex_pc_found.is_none() && regex_ubo_found.is_none() {
        txt = "
W_UBO_DEF{ float amoge;}
W_PC_DEF{ 
 UboObject ubo;         
}
             "
        .to_string()
          + &txt;
        // NOT FOUND
      }
      txt_clone = txt.clone();

      // so bad
      let mut regex_pc_found = regex_pc.find(&txt);
      let mut regex_ubo_found = regex_ubo.find(&txt_clone); // thefaq

      let mut push_constant_string = push_constant_string_upper.to_string();

      let mut rep_str_pc = push_constant_string_upper.to_string();
      match regex_pc_found {
        Some(_) => {
          rep_str_pc += " $1 ";
          rep_str_pc += push_constant_string_lower;
          txt = regex_pc.replace_all(&txt, rep_str_pc.as_str()).to_string();
        }
        None => {
          rep_str_pc += push_constant_content;
          rep_str_pc += push_constant_string_lower;
          if regex_ubo.find(&txt).is_none() {
            txt = rep_str_pc.clone() + &txt;
          }
        }
      }

      // -- UBO DIRECTIVE
      match regex_ubo_found {
        Some(_) => {
          let mut rep_str =
            "layout(buffer_reference, scalar, buffer_reference_align = 1, align = 1) readonly buffer UboObject { $1 };".to_string();
          txt = regex_ubo.replace_all(&txt, rep_str.as_str()).to_string();
        }
        None => {
          txt = "layout(buffer_reference, scalar, buffer_reference_align = 1, align = 1) readonly buffer UboObject { float amoge; };"
            .to_string()
            + &txt;
        }
      }

      // -- BUFF DIRECTIVE

      let regex_buff = regex::Regex::new(r"(?ms)W_BUFF_DEF(.*?)\{(.*?)\}").unwrap();

      // layout(set = 0, binding = 4, std430) buffer ${1}Buff { $1 buff; } ${1}_get[30]")
      // struct ${1} { $2 };
      txt = regex_buff
        .replace_all(
          &txt,
          "
          layout(set = 0, binding = 4, scalar, buffer_reference_align = 1, align = 1) buffer ${1}Buff { $2 } ${1}_get[]",
        )
        // layout(set = 0, binding = 4, std430) buffer ${1}Buff { $2 } ${1}_get[]"
        .to_string();
      //       let regex_buff = regex::Regex::new(r"(?ms)W_BUFF_DEF[ ]*\{(.*?)\}").unwrap();
      //       txt = regex_buff
      //           .replace_all(&txt, "
      // W_BUFF_DEF{ $1 }"
      //              ).to_string();

      txt = shared_import_string_glsl.to_string() + &shared_import_string_lower.to_string() + &txt;
    }

    // options.compile_into_spirv(source_text, shader_kind, input_file_name, entry_point_name, additional_options)

    let binary = {
      profiling::scope!("shaderc");
      compiler.compile_into_spirv(&txt, self.kind, &full_path, "main", Some(&options))
    };

    let mut err: String = String::from("");
    match binary {
      Ok(binary) => {
        profiling::scope!("binary to spv");
        let mut binaryu8 = binary.as_binary_u8();

        // let mut reflection = spirv_reflect::ShaderModule::load_u8_data(binaryu8);

        // let mut ep: String = wmemzeroed!();
        // let mut st: String = wmemzeroed!();
        // match reflection {
        //   #[allow(unused_assignments)]
        //   Ok(ref mut module) => {
        //     ep = module.get_entry_point_name();
        //     st = module.get_entry_point_name();
        //   }
        //   Err(_) => {
        //     debug_assert!(false)
        //   }
        // }

        let mut binaryu8 = std::io::Cursor::new(binaryu8);

        self.txt = txt;

        let vert_decoded = { 
          profiling::scope!("binary to vk");
          ash::util::read_spv(&mut binaryu8).unwrap()
        };
        let module_info = vk::ShaderModuleCreateInfo::builder().code(&vert_decoded);
        let shader_module = unsafe {
          profiling::scope!("module");
           device.create_shader_module(&module_info, None) 
         }.unwrap();


        {
          profiling::scope!("stage");

          // let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
          //   .stage(match self.kind {
          //     ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
          //     ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
          //     ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
          //     _ => vk::ShaderStageFlags::GEOMETRY,
          //   })
          //   .module(shader_module)
          //   .name(entry_point)
          //   .build();

          let stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(match self.kind {
              ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
              ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
              ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
              _ => vk::ShaderStageFlags::GEOMETRY,
            })
            .module(shader_module)
            .name(entry_point)
            .build();

          self.module.set(shader_module);
          self.stage.set(stage);

          self.compilation_error = String::from("");
        }
      }
      Err(__) => {
        self.compilation_error = __.to_string().clone();

        let mut line_idx = 1;
        for line in txt.lines() {
          println!("{}: {}", line_idx, line);
          line_idx = line_idx + 1;
        }
        println!("{}", self.compilation_error);
        // debug_assert!(false)
      }
    }
  }
}

pub struct WProgram {
  pub stages: SmallVec<[vk::PipelineShaderStageCreateInfo; 3]>,
  pub arena_idx: WAIdxShaderProgram,
  pub vert_shader: Option<WShader>,
  pub frag_shader: Option<WShader>,
  pub geom_shader: Option<WShader>,
  pub mesh_shader: Option<WShader>,
  pub comp_shader: Option<WShader>,
  pub vert_file_name: String,
  pub frag_file_name: String,
  pub comp_file_name: String,
}
unsafe impl Send for WProgram {}
impl WProgram {
  pub fn new_render_program(
    device: &ash::Device,
    folder: String,
    vert_file_name: String,
    frag_file_name: String,
  ) -> Self {
    let comp_file_name = "".to_string();

    unsafe {
      let vert_shader = WShader::new(device, ShaderKind::Vertex, folder.clone(), vert_file_name.clone());
      let frag_shader = WShader::new(device, ShaderKind::Fragment, folder.clone(), frag_file_name.clone());

      // https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Shader_modules
      // sussy bakki

      let mut stages = SmallVec::new();
      stages.push(vert_shader.stage.get());
      stages.push(frag_shader.stage.get());

      Self {
        stages,
        vert_shader: Some(vert_shader),
        frag_shader: Some(frag_shader),
        mesh_shader: None,
        geom_shader: None,
        comp_shader: None,
        vert_file_name,
        frag_file_name,
        comp_file_name,
        arena_idx: wmemzeroed!(),
      }
    }
  }

  pub fn refresh_program_stages(&mut self) {
    if let Some(frag_shader) = &self.frag_shader {
      self.stages[0] = frag_shader.stage.get();
    }

    if let Some(vert_shader) = &self.vert_shader {
      self.stages[1] = vert_shader.stage.get();
    }

    if let Some(comp_shader) = &self.comp_shader {
      self.stages[0] = comp_shader.stage.get();
    }
  }

  pub fn new_compute_program(
    device: &ash::Device,
    folder: String,
    shader_file_name: String,
  ) -> Self {
    unsafe {
      let vert_file_name = "".to_string();
      let frag_file_name = "".to_string();
      let comp_file_name = shader_file_name;

      let comp_shader = WShader::new(device, ShaderKind::Compute, folder, comp_file_name.clone());
      let mut stages = SmallVec::new();
      stages.push(comp_shader.stage.get());

      Self {
        stages,
        vert_shader: None,
        frag_shader: None,
        mesh_shader: None,
        geom_shader: None,
        comp_shader: Some(comp_shader),
        vert_file_name,
        frag_file_name,
        comp_file_name,
        arena_idx: wmemzeroed!(),
      }
    }
  }

  fn build(mut self) -> Self {
    self
  }
}
