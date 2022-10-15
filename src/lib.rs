#![allow(deref_nullptr)]
#![allow(non_snake_case)]
#![allow(unused_macros)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(invalid_value)]



#[macro_export]
macro_rules! wdef {
  () => {{
    Default::default()
  }}
}

#[macro_export]
macro_rules! wdyntoptr {
  ($x: expr, $t: ty ) => {{
    unsafe { &*($x as *const dyn Any as *const $t) }
  }};
}

#[macro_export]
macro_rules! wmemuninit {
  ( ) => {{
    unsafe { MaybeUninit::uninit().assume_init() }
  }};
}

#[macro_export]
macro_rules! wnullptr {
  ( ) => {{
    unsafe { std::ptr::null() }
  }};
}

#[macro_export]
macro_rules! ptralloc {
  ($t: ty ) => {unsafe{
    let layout = std::alloc::Layout::new::<$t>();
    let ptr= std::alloc::alloc(layout);
    ptr as *mut $t
  }};
}

#[macro_export]
macro_rules! w_ptr_to_mut_ref {
  ($p: expr ) => {unsafe{
    // let layout = std::alloc::Layout::new::<$t>();
    // let ptr= std::alloc::alloc(layout);
    // ptr as *mut $t
    &mut *$p
  }};
}

// #[macro_export]
// macro_rules! wcppnew {
//   (&v: expr, $t: ty ) => {unsafe{
//     let layout = std::alloc::Layout::new::<$t>();
//     let p= std::alloc::alloc(layout) as *mut $t;

//     std::ptr::write(p, $v);
//     p;
//   }};
// }




#[macro_export]
macro_rules! wtransmute {
  ($x: expr) => {{
    unsafe{
      std::mem::transmute($x)
    }
  }};
}

#[macro_export]
macro_rules! wptr {
  ($x: expr, $t: ty ) => {{
    unsafe{
        ($x as *const $t).as_ref().unwrap()
    }
  }};
}

#[macro_export]
macro_rules! wmemzeroed {
  ( ) => {{
    unsafe { std::mem::MaybeUninit::zeroed().assume_init() }
  }};
}
#[macro_export]
macro_rules! c_str {
    ($str:literal) => {
        unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($str, "\0").as_bytes()) }
    };
}


pub mod res;
pub mod sys;
pub mod abs;
pub mod wvulkan;
