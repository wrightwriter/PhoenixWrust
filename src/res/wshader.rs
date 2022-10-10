use std::cell::Cell;
use std::fs;
use std::mem::MaybeUninit;

use ash::vk::ShaderModule;


use shaderc::{self, ShaderKind};

use ash::vk;

use ash::{
    Device,
};

use std::ffi::CStr;

static entry_point: &'static CStr = unsafe { 
    CStr::from_bytes_with_nul_unchecked(b"main\0") 
};

// !! ---------- IMAGE ---------- //

enum ProgramType {
    Render,
    Compute
}

pub struct WShader{
    pub kind: ShaderKind,
    file_name: String,
    pub txt: String,
    pub stage: Cell<vk::PipelineShaderStageCreateInfo>,
    pub module: Cell<ShaderModule>,
    // pub binary: *mut u8
}

impl WShader {
    fn new(
        device: &Device,
        kind: ShaderKind,
        file_name: String,
    )->Self{
        let mut s = Self{
            kind: kind.clone(),
            file_name: file_name.clone(),
            txt: unsafe{MaybeUninit::zeroed().assume_init()},
            stage:unsafe{MaybeUninit::zeroed().assume_init()},
            module: unsafe{MaybeUninit::zeroed().assume_init()},
        };
        s.compile(
            device
        );
        s
    }
    fn compile(
        &mut self,
        device: &Device,
    ){
        let mut txt: String = unsafe{MaybeUninit::zeroed().assume_init()};

        match fs::read_to_string(self.file_name.clone()) {
            Ok(v) => {
                txt = v;
                Result::Ok(String::from(""))
                // txt= v.parse().unwrap()
            },
            Err(e) => {
                // assert!(false)
                debug_assert!(false);
                Result::Err(())
            }
        };

        let compiler = shaderc::Compiler::new().unwrap();

        let mut options = shaderc::CompileOptions::new().unwrap();
        shaderc::CompileOptions::set_generate_debug_info(&mut options);
        
        // options.compile_into_spirv(source_text, shader_kind, input_file_name, entry_point_name, additional_options)

        let binary = compiler.compile_into_spirv(
            &txt, self.kind, &self.file_name, "main", Some(&options)
        ).unwrap();

        let mut binaryu8 = binary.as_binary_u8();


        let mut reflection = spirv_reflect::ShaderModule::load_u8_data(binaryu8);

        let mut ep: String = wmemzeroed!();
        let mut st: String  = wmemzeroed!();
        match reflection {
            #[allow(unused_assignments)]
            Ok(ref mut module) => {
                ep = module.get_entry_point_name();
                st = module.get_entry_point_name();
            },
            Err(_) => {
                debug_assert!(false)
            },
        }

        let mut binaryu8 = std::io::Cursor::new(binaryu8);

        // let mut binaryu8 = binary.as_text();
        // self.binary.set(binary);

        self.txt = txt;


        let vert_decoded = ash::util::read_spv(&mut binaryu8).unwrap();
        let module_info = vk::ShaderModuleCreateInfo::builder().code(&vert_decoded);
        let shader_module = unsafe { device.create_shader_module(&module_info, None) }.unwrap();


        let vert_stage = 
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(
                    match self.kind {
                        ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
                        ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
                        ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
                        _ => vk::ShaderStageFlags::GEOMETRY
                    }
                    
                )
                .module(shader_module)
                .name( entry_point)
                .build()
                ;

        let vert_stage = 
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(
                    match self.kind {
                        ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
                        ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
                        ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
                        _ => vk::ShaderStageFlags::GEOMETRY
                    }
                    
                )
                .module(shader_module)
                .name( entry_point)
                .build()
                ;

        self.module.set(shader_module);
        self.stage.set( vert_stage);

    }
}


pub struct WProgram {
    pub stages: Vec<vk::PipelineShaderStageCreateInfo>,
    pub vert_shader: WShader,
    pub frag_shader: WShader,
    pub geom_shader: WShader,
    pub mesh_shader: WShader,
    pub comp_shader: WShader,
    pub vert_file_name: String,
    pub frag_file_name: String,
    pub comp_file_name: String,
}
unsafe impl Send for WProgram {}
impl WProgram{
    pub fn new_render_program(
      device: &ash::Device, 
      vert_file_name: String,
      frag_file_name: String,
    //   vert_string: &[u8],
    //   frag_string: &[u8],
    )->Self{
        let vert_file_name = vert_file_name;
        let frag_file_name = frag_file_name;
        let comp_file_name = "".to_string();

        unsafe{
            let vert_shader = WShader::new(device, ShaderKind::Vertex,vert_file_name.clone());
            let frag_shader = WShader::new(device,ShaderKind::Fragment,frag_file_name.clone());

            // https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Shader_modules
            // sussy bakki
            let stages = vec![
                vert_shader.stage.get(),
                frag_shader.stage.get(),
            ];

            Self{
                stages,
                vert_shader,
                frag_shader,
                mesh_shader: unsafe{MaybeUninit::uninit().assume_init()},
                geom_shader: unsafe{MaybeUninit::uninit().assume_init()},
                comp_shader: unsafe{MaybeUninit::uninit().assume_init()},
                vert_file_name,
                frag_file_name,
                comp_file_name,
            }
        }
    }
    pub fn new_compute_program(
      device: &ash::Device, 
      compute_file_name: String,
    )->Self{
        unsafe{
            let vert_file_name = "".to_string();
            let frag_file_name = "".to_string();
            let comp_file_name = compute_file_name;

            let comp_shader = WShader::new(device, ShaderKind::Compute,comp_file_name.clone());
            // https://vulkan-tutorial.com/Drawing_a_triangle/Graphics_pipeline_basics/Shader_modules
            let stages = vec![
                comp_shader.stage.get(),
            ];

            Self{
                stages,
                vert_shader: unsafe{MaybeUninit::uninit().assume_init()},
                frag_shader: unsafe{MaybeUninit::uninit().assume_init()},
                mesh_shader: unsafe{MaybeUninit::uninit().assume_init()},
                geom_shader: unsafe{MaybeUninit::uninit().assume_init()},
                comp_shader: comp_shader,
                vert_file_name,
                frag_file_name,
                comp_file_name,
            }
        }
    }

    fn build(mut self)->Self{

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