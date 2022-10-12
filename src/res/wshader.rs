use std::cell::Cell;
use std::fs;
use std::mem::MaybeUninit;

use ash::vk::ShaderModule;

use shaderc::{self, ShaderKind};

use ash::vk;

use ash::Device;
use smallvec::Array;
use smallvec::SmallVec;

use std::ffi::CStr;

use crate::sys::wdevice::GLOBALS;
use crate::sys::wmanagers::WAIdxComputePipeline;
use crate::sys::wmanagers::WAIdxRenderPipeline;

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
  folder: String,
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
      folder: folder,
      txt: unsafe { MaybeUninit::zeroed().assume_init() },
      stage: unsafe { MaybeUninit::zeroed().assume_init() },
      module: unsafe { MaybeUninit::zeroed().assume_init() },
      compilation_error: String::from(""),
      pipelines: SmallVec::new(),
    };
    s.try_compile(device);
    s
  }
  pub fn try_compile(
    &mut self,
    device: &Device,
  ) {
    let mut txt: String = unsafe { MaybeUninit::zeroed().assume_init() };

    let full_path = &(self.folder.clone() + &self.file_name);

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

    let mut options = shaderc::CompileOptions::new().unwrap();
    shaderc::CompileOptions::set_generate_debug_info(&mut options);

    let dont_preprocess_regex = regex::Regex::new(r"\#W_DONT_PREPROCESS").unwrap();
    if let Some(__) = dont_preprocess_regex
      .find(&txt)
    {
      txt = dont_preprocess_regex.replace(&txt, "").to_string();
    } else {
      let shared_import_string_glsl = "#version 450 core
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_buffer_reference : require
#extension GL_EXT_buffer_reference2 : require
#extension GL_EXT_buffer_reference_uvec2 : require
#extension GL_EXT_scalar_block_layout : enable
      ";

  let shared_import_string_lower = "
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


layout( push_constant ) uniform constants{
  Amogus ubo;
  int frame;
} PC;


layout(set = 0, binding=0) uniform SharedUbo{
    vec4 values[];
} shared_ubo; 

layout(rgba32f, set = 0, binding = 1) uniform image2D shared_images[10];

      ";
      let mut shared_import_string = shared_import_string_glsl.to_string();

      // UBO DIRECTIVE
      let regex_ubo = regex::Regex::new(r"(?ms)W_UBO_DEF\{(.*?)\}").unwrap();

      let regex_ubo_found = regex_ubo.find(&txt);

      match regex_ubo_found {
        Some(_) => {
          let rep_str = "layout(buffer_reference, scalar, buffer_reference_align = 1, align = 1) readonly buffer Amogus { $1 };".to_string() 
                  + &shared_import_string_lower.to_string();
          let rep_str = rep_str.as_str();

          txt = regex_ubo.replace_all(&txt, rep_str).to_string();
        }
        None => {
          txt = "layout(buffer_reference, scalar, buffer_reference_align = 1, align = 1) readonly buffer Amogus { float amoge; };".to_string() 
                + &shared_import_string_lower.to_string()
                + &txt;
        }
      }

      // txt = shared_import_string.to_string() + &txt;
      txt = shared_import_string_glsl.to_string() + &txt;
    }

    // W_UBO_DEF{.*?}

    // options.compile_into_spirv(source_text, shader_kind, input_file_name, entry_point_name, additional_options)

    let binary = compiler.compile_into_spirv(&txt, self.kind, &full_path, "main", Some(&options));

    let mut err: String = String::from("");
    match binary {
      Ok(binary) => {
        let mut binaryu8 = binary.as_binary_u8();

        let mut reflection = spirv_reflect::ShaderModule::load_u8_data(binaryu8);

        let mut ep: String = wmemzeroed!();
        let mut st: String = wmemzeroed!();
        match reflection {
          #[allow(unused_assignments)]
          Ok(ref mut module) => {
            ep = module.get_entry_point_name();
            st = module.get_entry_point_name();
          }
          Err(_) => {
            debug_assert!(false)
          }
        }

        let mut binaryu8 = std::io::Cursor::new(binaryu8);

        // let mut binaryu8 = binary.as_text();
        // self.binary.set(binary);

        self.txt = txt;

        let vert_decoded = ash::util::read_spv(&mut binaryu8).unwrap();
        let module_info = vk::ShaderModuleCreateInfo::builder().code(&vert_decoded);
        let shader_module = unsafe { device.create_shader_module(&module_info, None) }.unwrap();

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
          .stage(match self.kind {
            ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
            ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
            ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
            _ => vk::ShaderStageFlags::GEOMETRY,
          })
          .module(shader_module)
          .name(entry_point)
          .build();

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
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
        self.stage.set(vert_stage);

        self.compilation_error = String::from("");
      }
      Err(__) => {
        self.compilation_error = __.to_string().clone();
        debug_assert!(false)
      }
    }
  }
}

pub struct WProgram {
  pub stages: SmallVec<[vk::PipelineShaderStageCreateInfo; 3]>,
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
    //   vert_string: &[u8],
    //   frag_string: &[u8],
  ) -> Self {
    // let vert_file_name = vert_file_name;
    // let frag_file_name = frag_file_name;
    let comp_file_name = "".to_string();

    unsafe {
      let vert_shader = WShader::new(
        device,
        ShaderKind::Vertex,
        folder.clone(),
        vert_file_name.clone(),
      );
      let frag_shader = WShader::new(
        device,
        ShaderKind::Fragment,
        folder.clone(),
        frag_file_name.clone(),
      );

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
      }
    }
  }

  pub fn refresh_program_stages(&mut self) {
    // unsafe{
    //   self.stages.set_len(0)
    // }

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
      // https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Shader_modules
      // let stages = vec![comp_shader.stage.get()];
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
      }
    }
  }

  fn build(mut self) -> Self {
    self
  }
}

// impl Default for WImage{
//     fn default() -> Self {
//         Self { handle: None, resx: 500, resy: 500, format: None, view: None }
//     }
// }

// unsafe {std::ffi::CStr::from_bytes_with_nul_unchecked(buffer.as_bytes())}

// __entry_point = Some(CString::new(*b"asg_\n\0").unwrap());
// let ep = CString::new("main").unwrap();
// let ep = CString::new("main").unwrap();

// __entry_point = CStr::from_bytes_with_nul(b"main\n\0").unwrap();
// let t = CString::new("main").unwrap();
// __entry_point = CStr::from_bytes_with_nul_unchecked(t.to_bytes_with_nul());
// let entry_point = CStr::from_bytes_with_nul_unchecked(CString::new("main").unwrap().to_bytes_with_nul());

// let entry_point = CString::new(*b"asg_\n\0").unwrap().as_c_str();
// let entry_point = CString::new("main").unwrap();

// let bb = CStr::from_bytes_until_nul(entry_point);
// let bb = CStr::from_ptr(entry_point as *const u8);
// let bb = CStr::from(b"main\n\0");
// _entry_point = CStr::new("main").unwrap();

// let _entry_point = unsafe{(entry_point.as_ptr() as *const CString)};
// let _entry_point = "main";

// static mut entry_point: CString = CString::new("main").unwrap();
