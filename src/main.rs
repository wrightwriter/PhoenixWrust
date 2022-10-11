#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_must_use)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(invalid_value)]

use phoenix_wrust::{wvulkan::WVulkan, ptralloc, sys::wdevice::GLOBALS};
use winit::event_loop::EventLoop;

extern crate spirv_reflect;

fn main() {
  // let w_render_doc:RenderDoc<V141> = RenderDoc::new().expect("Unable to set up renderdoc");

  let event_loop: EventLoop<()>;
  event_loop = EventLoop::new();

  let window = WVulkan::init_window(&event_loop);

  unsafe{
    GLOBALS.w_vulkan = ptralloc!(WVulkan);
    std::ptr::write(GLOBALS.w_vulkan, WVulkan::new(&window));


    // GLOBALS.w_vulkan.borrow().run(event_loop, &window);
    WVulkan::run(event_loop, &window);

  }
}
