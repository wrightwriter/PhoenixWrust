// #![allow(unused)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_unsafe)]
#![allow(unused_must_use)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(invalid_value)]


#[macro_export]
macro_rules! wdef {
  () => {{
    // unsafe { &*($x as *const dyn Any as *const $t) }
    Default::default()
    // unsafe { &*($x as *const dyn Any as *const $t) }
  }}
}

#[macro_export]
macro_rules! wdyntoptr {
  ($x: expr, $t: ty ) => {{
    // unsafe { &*($x as *const dyn Any as *const $t) }
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
    unsafe { MaybeUninit::zeroed().assume_init() }
  }};
}
#[macro_export]
macro_rules! c_str {
    ($str:literal) => {
        unsafe { std::ffi::CStr::from_bytes_with_nul_unchecked(concat!($str, "\0").as_bytes()) }
    };
}

// pub use res::wbuffer;
// use res;

pub mod res;
pub mod sys;

pub mod macros;

pub mod abs;

// pub type VkResult<T> = Result<T, VkError>;

// #[derive(Debug)]
// pub enum VkError {
//   Entry(EntryError),
//   Loader(LoaderError),
//   Vk(vk::Result),
//   Image(ImageError),
//   ObjLoad(LoadError),
//   ValidationLayerUnavailable,
//   NoVulkanGpu,
//   NoSuitableGpu,
//   NoSuitableMemoryType,
//   NoSupportedFormat,
//   UnsupportedLayoutTransition,
//   UnsupportedLinearBlitting,
// }

// impl From<EntryError> for VkError {
//   fn from(err: EntryError) -> Self {
//     Self::Entry(err)
//   }
// }

// impl From<LoaderError> for VkError {
//   fn from(err: LoaderError) -> Self {
//     Self::Loader(err)
//   }
// }

// impl From<vk::Result> for VkError {
//   fn from(err: vk::Result) -> Self {
//     Self::Vk(err)
//   }
// }

// impl From<ImageError> for VkError {
//   fn from(err: ImageError) -> Self {
//     Self::Image(err)
//   }
// }

// impl From<LoadError> for VkError {
//   fn from(err: LoadError) -> Self {
//     Self::ObjLoad(err)
//   }
// }

// impl fmt::Display for VkError {
//   fn fmt(
//     &self,
//     f: &mut fmt::Formatter,
//   ) -> fmt::Result {
//     match self {
//       VkError::Entry(_) => f.write_str("entry loader error"),
//       VkError::Loader(_) => f.write_str("loader error"),
//       VkError::Vk(err) => write!(f, "vulkan error {}", err.0),
//       VkError::Image(_) => f.write_str("image error"),
//       VkError::ObjLoad(_) => f.write_str("obj load error"),
//       VkError::ValidationLayerUnavailable => {
//         f.write_str("validation layers requested, but not available")
//       }
//       VkError::NoVulkanGpu => f.write_str("failed to find GPUs with Vulkan support"),
//       VkError::NoSuitableGpu => f.write_str("failed to find a suitable GPU"),
//       VkError::NoSuitableMemoryType => f.write_str("failed to find suitable memory type"),
//       VkError::NoSupportedFormat => f.write_str("failed to find supported format"),
//       VkError::UnsupportedLayoutTransition => f.write_str("unsupported layout transition"),
//       VkError::UnsupportedLinearBlitting => {
//         f.write_str("texture image format does not support linear blitting!")
//       }
//     }
//   }
// }

// impl Error for VkError {
//   fn source(&self) -> Option<&(dyn Error + 'static)> {
//     match self {
//       VkError::Entry(err) => Some(err),
//       VkError::Loader(err) => Some(err),
//       VkError::Vk(err) => Some(err),
//       VkError::Image(err) => Some(err),
//       VkError::ObjLoad(err) => Some(err),
//       VkError::ValidationLayerUnavailable
//       | VkError::NoVulkanGpu
//       | VkError::NoSuitableGpu
//       | VkError::NoSuitableMemoryType
//       | VkError::NoSupportedFormat
//       | VkError::UnsupportedLayoutTransition
//       | VkError::UnsupportedLinearBlitting => None,
//     }
//   }
// }
