#![allow(unused_macros)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_must_use)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(invalid_value)]


use std::{ffi::c_int, i32};

use phoenix_wrust::{wvulkan::WVulkan, ptralloc, sys::wdevice::GLOBALS, msdf::msdf::WFont};
use tracy_client::span;
use winit::event_loop::EventLoop;

extern crate spirv_reflect;


// #[link(name = "wffmpeg")]
// extern "C" {
//   pub fn test() -> i32;
// }

fn main() {
    // std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\videos\\";
  std::env::set_var("WORKSPACE_DIR", "D:\\Programming\\Demoscene\\PhoenixWrust");
  std::env::set_var("FFMPEG_DIR", "D:\\Programming\\Demoscene\\PhoenixWrust\\ffmpeg");
  std::env::set_var("LIBCLANG_PATH", "C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\VC\\Tools\\Llvm\\x64\\bin\\");
  
  // let w_render_doc:RenderDoc<V141> = RenderDoc::new().expect("Unable to set up renderdoc");
  
  // unsafe{
  //   let a = test();
  // }
    
  #[cfg(not(debug_assertions))]
  {
    let cwd = std::env::current_dir().unwrap();
    let cwd = cwd.to_str().unwrap();
    std::env::set_var("WORKSPACE_DIR", cwd);
  }
  #[cfg(debug_assertions)]
  let tracy = tracy_client::Client::start();
  #[cfg(debug_assertions)]
  profiling::register_thread!("Main Thread");

  // std::env::var().unwrap() + "\\src\\models\\";

  let event_loop: EventLoop<()>;
  event_loop = EventLoop::new();

  let window = WVulkan::init_window(&event_loop);

  unsafe{
    GLOBALS.w_vulkan = ptralloc!(WVulkan);
    std::ptr::write(GLOBALS.w_vulkan, WVulkan::new(&window));

    WVulkan::run(event_loop, &window);
  }
}
