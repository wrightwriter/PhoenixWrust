use crate::res;
use crate::res::wshader::{WProgram, WShaderEnumPipelineBind};
use crate::sys::warenaitems::WAIdxShaderProgram;
use crate::sys::wdevice::{WDevice, GLOBALS};
use generational_arena::Arena;
use notify::{Error, Event, ReadDirectoryChangesWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use smallvec::SmallVec;



use std::borrow::BorrowMut;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use tracy_client::span;

use crate::sys::warenaitems::WAIdxRenderPipeline;

use crate::sys::warenaitems::WArenaItem;






pub struct WShaderMan {
  pub root_shader_dir: String,
  pub shader_was_modified: Arc<Mutex<bool>>,

  pub shaders_with_errors: Arc<Mutex<Vec<WAIdxShaderProgram>>>,
  pub pipelines_with_errors: Arc<Mutex<Vec<WAIdxRenderPipeline>>>,

  watcher: ReadDirectoryChangesWatcher,

  pub chan_sender_start_shader_comp: Sender<()>,
  pub chan_receiver_end_shader_comp: Receiver<()>,
}

impl WShaderMan {
  pub fn new() -> Self {
    let root_shader_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\shaders\\";
    let root_shader_dir = Self::sanitize_path(root_shader_dir);

    let rsd = root_shader_dir.clone();

    let shader_was_modified = Arc::new(Mutex::new(false));
    let shader_was_modified_clone = shader_was_modified.clone();

    let shaders_with_errors: Arc<Mutex<Vec<WAIdxShaderProgram>>> = Arc::new(Mutex::new(Vec::new()));
    let shaders_with_errors_clone = shaders_with_errors.clone();

    let pipelines_with_errors = Arc::new(Mutex::new(Vec::new()));
    let pipelines_with_errors_clone = pipelines_with_errors.clone();

    let (chan_sender_start_shader_comp, chan_receiver_start_shader_comp) = channel();
    let (chan_sender_end_shader_comp, chan_receiver_end_shader_comp) = channel();

    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    unsafe {
      let comp = Box::new(shaderc::Compiler::new().unwrap());
      let comp = Box::into_raw(comp);
      GLOBALS.compiler = comp;
    }

    unsafe {
      GLOBALS.shader_programs_arena = ptralloc!(Arena<WProgram>);
      std::ptr::write(GLOBALS.shader_programs_arena, Arena::new());
    };

    let mut watcher = RecommendedWatcher::new(
      move |result: Result<Event, Error>| {
        profiling::scope!("shader reload");
        let event = result.unwrap();

        *shader_was_modified_clone.lock().unwrap() = true;
        chan_receiver_start_shader_comp.recv().expect("Error: timed out.");

        {
          let mut cnt = counter_clone.lock().unwrap();
          let cnt_val = *cnt;
          if cnt_val < 2{
            *cnt += 1;
            *shader_was_modified_clone.lock().unwrap() = false;
            chan_sender_end_shader_comp.send(());
            return;
          } else {
            *cnt = 0;
          }
        }

        profiling::scope!("shader watcher");
        if event.kind.is_modify() {
          for __ in &event.paths {
            let mut path = __.as_os_str().to_str().unwrap();
            let mut path = String::from(path);
            path = Self::sanitize_path(path);
            path = path.replace(&root_shader_dir, "");

            let mut pipelines_which_need_reloading: SmallVec<[WShaderEnumPipelineBind; 10]> = SmallVec::new();

            macro_rules! reload_shader {
              ($shader: expr ) => {
                unsafe {
                  // true
                  if ($shader.file_name == path) {
                    $shader.try_compile(unsafe { &(&*GLOBALS.w_vulkan).w_device.device });

                    println!("-- SHADER RELOAD -- {}", path);

                    if ($shader.compilation_error != "") {
                      println!("{}", $shader.compilation_error);
                      false
                    } else {
                      for pipeline in &$shader.pipelines {
                        pipelines_which_need_reloading.push(*pipeline)
                      }
                      true
                    }
                  } else {
                    true
                  }
                }
              };
            }

            unsafe {
              for shader_program in &mut *GLOBALS.shader_programs_arena {
                let mut success = true;
                if let Some(comp_shader) = &mut shader_program.1.comp_shader {
                  let sc = reload_shader!(comp_shader);
                  if !sc {
                    success = false;
                  }
                } else {
                  if let Some(frag_shader) = &mut shader_program.1.frag_shader {
                    let sc = reload_shader!(frag_shader);
                    if !sc {
                      success = false;
                    }
                  }
                  if let Some(vert_shader) = &mut shader_program.1.vert_shader {
                    let sc = reload_shader!(vert_shader);
                    if !sc {
                      success = false;
                    }
                  } 
                }

                if !success {
                  let shader_programs_with_errors = &mut *shaders_with_errors_clone.lock().unwrap();

                  let mut found_shader = false;
                  {
                    for other_shader_prog in &mut *shader_programs_with_errors {
                      if other_shader_prog.idx.clone() == shader_program.0 {
                        found_shader = true;
                      }
                    }
                  }
                  if !found_shader {
                    shader_programs_with_errors.push(shader_program.1.arena_idx);
                  }
                } else {
                  let shader_programs_with_errors = &mut *shaders_with_errors_clone.lock().unwrap();
                  shader_programs_with_errors.retain(|&prog| prog.idx != shader_program.0);
                }
              }
            }

            macro_rules! refresh_pipeline {
              ($pipeline: expr ) => {
                unsafe {
                  {
                    $pipeline.get_mut().shader_program.get_mut().refresh_program_stages();
                  }
                  $pipeline
                    .get_mut()
                    // .refresh_pipeline(&(*GLOBALS.w_vulkan).w_device.device, &(*GLOBALS.w_vulkan).w_grouper);
                    .refresh_pipeline(&(*GLOBALS.w_vulkan).w_device.device, &(*GLOBALS.w_tl));
                }
              };
            }

            {
              profiling::scope!("pipeline reload");
              for pipeline in pipelines_which_need_reloading {
                match pipeline {
                  res::wshader::WShaderEnumPipelineBind::ComputePipeline(pipeline) => unsafe {
                    refresh_pipeline!(pipeline);
                  },
                  res::wshader::WShaderEnumPipelineBind::RenderPipeline(pipeline) => unsafe {
                    refresh_pipeline!(pipeline);
                  },
                }
              }
            }
          }
        }
        *shader_was_modified_clone.lock().unwrap() = false;
        chan_sender_end_shader_comp.send(());
      },
      notify::Config::default(),
    )
    .unwrap();

    watcher.watch(Path::new(&rsd), RecursiveMode::Recursive).unwrap();

    Self {
      root_shader_dir: rsd,
      shader_was_modified,
      watcher,
      chan_sender_start_shader_comp,
      chan_receiver_end_shader_comp,
      shaders_with_errors,
      pipelines_with_errors,
    }
  }

  fn sanitize_path(path: String) -> String {
    let re = regex::Regex::new(r"/").unwrap().replace_all(&path, "\\").to_string();

    re
  }
  // TODO: move to TL
  pub fn new_render_program<S: Into<String>>(
    &mut self,
    w_device: &mut WDevice,
    mut vert_file_name: S,
    mut frag_file_name: S,
  ) -> WAIdxShaderProgram {
    let vert_file_name = Self::sanitize_path(vert_file_name.into());
    let frag_file_name = Self::sanitize_path(frag_file_name.into());

    let idx = unsafe {
      (*GLOBALS.shader_programs_arena).insert(WProgram::new_render_program(
        &w_device.device,
        self.root_shader_dir.clone(),
        vert_file_name,
        frag_file_name,
      ))
    };
    let idx = WAIdxShaderProgram { idx };

    let sp = idx.get_mut();
    sp.arena_idx = idx;

    idx
  }

  pub fn new_compute_program<S: Into<String>>(
    &mut self,
    w_device: &mut WDevice,
    mut compute_file_name: S,
  ) -> WAIdxShaderProgram {
    let compute_file_name = Self::sanitize_path(compute_file_name.into());

    let idx = unsafe {
      (*GLOBALS.shader_programs_arena).insert(WProgram::new_compute_program(
        &w_device.device,
        self.root_shader_dir.clone(),
        compute_file_name,
      ))
    };
    let idx = WAIdxShaderProgram { idx };

    let sp = idx.get_mut();
    sp.arena_idx = idx;

    idx
  }
}
