use ash::vk;


pub enum WFormatType {
  SNORM,
  UNORM,
  UINT,
  SINT,
  SRGB,
  SFLOAT,
}

pub trait WFormatTools{
  fn chan_cnt(&self)->u32;
  fn get_type(&self)->WFormatType;
  fn bits_per_chan(&self)->u32;
  fn bytes_per_chan(&self)->u32;
}



impl WFormatTools for vk::Format {
    fn bytes_per_chan(&self)->u32{
      self.bits_per_chan()/8
    }

    fn bits_per_chan(&self)->u32{ 
      let fm = *self;
      if(
        vk::Format::R8_UNORM == fm ||
        vk::Format::R8G8B8A8_UNORM == fm ||
        vk::Format::B8G8R8_UNORM == fm ||
        vk::Format::R8G8B8_UNORM == fm ||
        vk::Format::B8G8R8A8_UNORM == fm ||
        vk::Format::A8B8G8R8_UNORM_PACK32 == fm ||
        vk::Format::R8G8_UNORM == fm ||
        vk::Format::R8G8B8_UINT == fm ||
        vk::Format::B8G8R8A8_SNORM == fm ||
        vk::Format::R8G8B8A8_SNORM == fm ||
        vk::Format::B8G8R8_SNORM == fm ||
        vk::Format::R8G8B8_SNORM == fm ||
        vk::Format::R8G8_SNORM == fm ||
        vk::Format::R8_SNORM == fm ||
        vk::Format::A8B8G8R8_UINT_PACK32 == fm ||
        vk::Format::B8G8R8A8_UINT == fm ||
        vk::Format::R8G8B8A8_UINT == fm ||
        vk::Format::B8G8R8_UINT == fm ||
        vk::Format::R8G8_UINT == fm ||
        vk::Format::R8_UINT == fm ||
        vk::Format::A8B8G8R8_SNORM_PACK32 == fm ||
        vk::Format::B8G8R8A8_SINT == fm ||
        vk::Format::R8G8B8A8_SINT == fm ||
        vk::Format::B8G8R8_SINT == fm ||
        vk::Format::R8G8B8_SINT == fm ||
        vk::Format::R8G8_SINT == fm ||
        vk::Format::R8_SINT == fm 
      ){
        return 8;
      } else if(
        vk::Format::R16G16_UNORM == fm ||
        vk::Format::R16G16B16_UNORM == fm ||
        vk::Format::R16G16B16A16_UNORM == fm ||
        vk::Format::R16_UNORM == fm ||
        vk::Format::R16G16B16A16_SNORM == fm ||
        vk::Format::R16_SFLOAT == fm ||
        vk::Format::R16G16_SFLOAT == fm ||
        vk::Format::R16G16B16_SFLOAT == fm || 
        vk::Format::R16G16B16A16_SFLOAT == fm ||
        vk::Format::R16G16B16_SINT == fm ||
        vk::Format::R16G16_SINT == fm ||
        vk::Format::R16_SINT == fm ||
        vk::Format::R16G16B16A16_SINT == fm ||
        vk::Format::R16G16B16_UINT == fm ||
        vk::Format::R16G16_UINT == fm ||
        vk::Format::R16_UINT == fm ||
        vk::Format::R16G16B16A16_UINT == fm ||
        vk::Format::R16G16B16_SNORM == fm ||
        vk::Format::R16G16_SNORM == fm ||
        vk::Format::R16_SNORM == fm 
      ){
        return 16;
      } else if(
        vk::Format::R32_SINT == fm ||
        vk::Format::R32_SFLOAT == fm ||
        vk::Format::R32G32_SFLOAT == fm || 
        vk::Format::R32G32B32_SFLOAT == fm  ||
        vk::Format::R32G32B32A32_SFLOAT == fm ||
        vk::Format::R32G32B32_SINT == fm ||
        vk::Format::R32G32B32A32_UINT == fm ||
        vk::Format::R32G32B32A32_SINT == fm ||
        vk::Format::R32_UINT == fm ||
        vk::Format::R32G32B32_UINT == fm ||
        vk::Format::R32G32_UINT == fm ||
        vk::Format::R32G32_SINT == fm
      ){
        return 32;
      } else {
      // if(
        // vk::Format::A2B10G10R10_UINT_PACK32 == fm ||
        // vk::Format::A2R10G10B10_UINT_PACK32 == fm ||
        // vk::Format::A2B10G10R10_SINT_PACK32 == fm ||
        // vk::Format::A2R10G10B10_SINT_PACK32 == fm ||
        // vk::Format::A8B8G8R8_SINT_PACK32 == fm ||
        // vk::Format::A2B10G10R10_SNORM_PACK32 == fm ||
        // vk::Format::A2R10G10B10_SNORM_PACK32 == fm ||
        // vk::Format::A2R10G10B10_UNORM_PACK32 == fm ||
        // vk::Format::A2B10G10R10_UNORM_PACK32 == fm 
        // vk::Format::R8_USCALED == fm ||
        // vk::Format::R8_SSCALED == fm ||
        // vk::Format::R8_SRGB == fm ||
        // vk::Format::R16_USCALED == fm ||
        // vk::Format::R16_SSCALED == fm ||
        // vk::Format::R8G8_USCALED == fm ||
        // vk::Format::R8G8_SSCALED == fm ||
        // vk::Format::R8G8_SRGB == fm ||
        // vk::Format::R16G16_USCALED == fm ||
        // vk::Format::R16G16_SSCALED == fm ||
        // vk::Format::R8G8B8_USCALED == fm ||
        // vk::Format::R8G8B8_SSCALED == fm ||
        // vk::Format::R8G8B8_SRGB == fm ||
        // vk::Format::B8G8R8_USCALED == fm ||
        // vk::Format::B8G8R8_SSCALED == fm ||
        // vk::Format::B8G8R8_SRGB == fm ||
        // vk::Format::R16G16B16_USCALED == fm ||
        // vk::Format::R16G16B16_SSCALED == fm ||
        // vk::Format::R8G8B8A8_USCALED == fm ||
        // vk::Format::R8G8B8A8_SSCALED == fm ||
        // vk::Format::R8G8B8A8_SRGB == fm ||
        // vk::Format::B8G8R8A8_USCALED == fm ||
        // vk::Format::B8G8R8A8_SSCALED == fm ||
        // vk::Format::B8G8R8A8_SRGB == fm ||
        // vk::Format::A8B8G8R8_USCALED_PACK32 == fm ||
        // vk::Format::A8B8G8R8_SSCALED_PACK32 == fm ||
        // vk::Format::A8B8G8R8_SRGB_PACK32 == fm ||
        // vk::Format::A2R10G10B10_USCALED_PACK32 == fm ||
        // vk::Format::A2R10G10B10_SSCALED_PACK32 == fm ||
        // vk::Format::A2B10G10R10_USCALED_PACK32 == fm ||
        // vk::Format::A2B10G10R10_SSCALED_PACK32 == fm ||
        // vk::Format::R16G16B16A16_USCALED == fm ||
        // vk::Format::R16G16B16A16_SSCALED == fm ||
      // )
        panic!();
        return 0;
      }

    }

    fn get_type(&self)->WFormatType { 
      let fm = *self;
      if(
        vk::Format::R8_UNORM == fm ||
        vk::Format::R8G8B8A8_UNORM == fm ||
        vk::Format::B8G8R8_UNORM == fm ||
        vk::Format::R16G16_UNORM == fm ||
        vk::Format::R8G8B8_UNORM == fm ||
        vk::Format::R16G16B16_UNORM == fm ||
        vk::Format::B8G8R8A8_UNORM == fm ||
        vk::Format::A8B8G8R8_UNORM_PACK32 == fm ||
        vk::Format::A2R10G10B10_UNORM_PACK32 == fm ||
        vk::Format::A2B10G10R10_UNORM_PACK32 == fm ||
        vk::Format::R16G16B16A16_UNORM == fm ||
        vk::Format::R16_UNORM == fm ||
        vk::Format::R8G8_UNORM == fm 
      ){
        return WFormatType::UNORM;
      } else if(
        vk::Format::R16G16B16A16_SNORM == fm ||
        vk::Format::A2B10G10R10_SNORM_PACK32 == fm ||
        vk::Format::A2R10G10B10_SNORM_PACK32 == fm ||
        vk::Format::A8B8G8R8_SNORM_PACK32 == fm ||
        vk::Format::B8G8R8A8_SNORM == fm ||
        vk::Format::R8G8B8A8_SNORM == fm ||
        vk::Format::R16G16B16_SNORM == fm ||
        vk::Format::B8G8R8_SNORM == fm ||
        vk::Format::R8G8B8_SNORM == fm ||
        vk::Format::R16G16_SNORM == fm ||
        vk::Format::R8G8_SNORM == fm ||
        vk::Format::R8_SNORM == fm ||
        vk::Format::R16_SNORM == fm 
      ){
        return WFormatType::SNORM;
      } else if(
        vk::Format::R32G32B32A32_UINT == fm ||
        vk::Format::R16G16B16A16_UINT == fm ||
        vk::Format::A2B10G10R10_UINT_PACK32 == fm ||
        vk::Format::A2R10G10B10_UINT_PACK32 == fm ||
        vk::Format::A8B8G8R8_UINT_PACK32 == fm ||
        vk::Format::B8G8R8A8_UINT == fm ||
        vk::Format::R8G8B8A8_UINT == fm ||
        vk::Format::R32G32B32_UINT == fm ||
        vk::Format::B8G8R8_UINT == fm ||
        vk::Format::R16G16B16_UINT == fm ||
        vk::Format::R8G8B8_UINT == fm ||
        vk::Format::R32G32_UINT == fm ||
        vk::Format::R16G16_UINT == fm ||
        vk::Format::R8G8_UINT == fm ||
        vk::Format::R32_UINT == fm ||
        vk::Format::R16_UINT == fm ||
        vk::Format::R8_UINT == fm 
      ){
        return WFormatType::UINT;
      } else if(
        vk::Format::R32G32B32A32_SINT == fm ||
        vk::Format::R16G16B16A16_SINT == fm ||
        vk::Format::A2B10G10R10_SINT_PACK32 == fm ||
        vk::Format::A2R10G10B10_SINT_PACK32 == fm ||
        vk::Format::A8B8G8R8_SINT_PACK32 == fm ||
        vk::Format::B8G8R8A8_SINT == fm ||
        vk::Format::R8G8B8A8_SINT == fm ||
        vk::Format::R32G32B32_SINT == fm ||
        vk::Format::R16G16B16_SINT == fm ||
        vk::Format::B8G8R8_SINT == fm ||
        vk::Format::R8G8B8_SINT == fm ||
        vk::Format::R32G32_SINT == fm ||
        vk::Format::R16G16_SINT == fm ||
        vk::Format::R8G8_SINT == fm ||
        vk::Format::R32_SINT == fm ||
        vk::Format::R16_SINT == fm ||
        vk::Format::R8_SINT == fm 
      ){
        return WFormatType::SINT;
      } else if(
        vk::Format::R16_SFLOAT == fm ||
        vk::Format::R32_SFLOAT == fm ||
        vk::Format::R16G16_SFLOAT == fm ||
        vk::Format::R32G32_SFLOAT == fm || 
        vk::Format::R16G16B16_SFLOAT == fm || 
        vk::Format::R32G32B32_SFLOAT == fm  ||
        vk::Format::R16G16B16A16_SFLOAT == fm ||
        vk::Format::R32G32B32A32_SFLOAT == fm
        // vk::Format::R8_USCALED == fm ||
        // vk::Format::R8_SSCALED == fm ||
        // vk::Format::R8_SRGB == fm ||
        // vk::Format::R16_USCALED == fm ||
        // vk::Format::R16_SSCALED == fm ||
        // vk::Format::R8G8_USCALED == fm ||
        // vk::Format::R8G8_SSCALED == fm ||
        // vk::Format::R8G8_SRGB == fm ||
        // vk::Format::R16G16_USCALED == fm ||
        // vk::Format::R16G16_SSCALED == fm ||
        // vk::Format::R8G8B8_USCALED == fm ||
        // vk::Format::R8G8B8_SSCALED == fm ||
        // vk::Format::R8G8B8_SRGB == fm ||
        // vk::Format::B8G8R8_USCALED == fm ||
        // vk::Format::B8G8R8_SSCALED == fm ||
        // vk::Format::B8G8R8_SRGB == fm ||
        // vk::Format::R16G16B16_USCALED == fm ||
        // vk::Format::R16G16B16_SSCALED == fm ||
        // vk::Format::R8G8B8A8_USCALED == fm ||
        // vk::Format::R8G8B8A8_SSCALED == fm ||
        // vk::Format::R8G8B8A8_SRGB == fm ||
        // vk::Format::B8G8R8A8_USCALED == fm ||
        // vk::Format::B8G8R8A8_SSCALED == fm ||
        // vk::Format::B8G8R8A8_SRGB == fm ||
        // vk::Format::A8B8G8R8_USCALED_PACK32 == fm ||
        // vk::Format::A8B8G8R8_SSCALED_PACK32 == fm ||
        // vk::Format::A8B8G8R8_SRGB_PACK32 == fm ||
        // vk::Format::A2R10G10B10_USCALED_PACK32 == fm ||
        // vk::Format::A2R10G10B10_SSCALED_PACK32 == fm ||
        // vk::Format::A2B10G10R10_USCALED_PACK32 == fm ||
        // vk::Format::A2B10G10R10_SSCALED_PACK32 == fm ||
        // vk::Format::R16G16B16A16_USCALED == fm ||
        // vk::Format::R16G16B16A16_SSCALED == fm ||
      ){
        return WFormatType::SFLOAT;
      } else {
        panic!();
        return WFormatType::SRGB;
      }
    }

    fn chan_cnt(&self)-> u32{
      let fm = *self;
      if(
        vk::Format::R8_UNORM == fm ||
        vk::Format::R8_SNORM == fm ||
        vk::Format::R8_USCALED == fm ||
        vk::Format::R8_SSCALED == fm ||
        vk::Format::R8_UINT == fm ||
        vk::Format::R8_SINT == fm ||
        vk::Format::R8_SRGB == fm ||
        vk::Format::R16_UNORM == fm ||
        vk::Format::R16_SNORM == fm ||
        vk::Format::R16_USCALED == fm ||
        vk::Format::R16_SSCALED == fm ||
        vk::Format::R16_UINT == fm ||
        vk::Format::R16_SINT == fm ||
        vk::Format::R16_SFLOAT == fm ||
        vk::Format::R32_UINT == fm ||
        vk::Format::R32_SINT == fm ||
        vk::Format::R32_SFLOAT == fm 
      ){
        return 1;
      } else if(
        vk::Format::R8G8_UNORM == fm ||
        vk::Format::R8G8_SNORM == fm ||
        vk::Format::R8G8_USCALED == fm ||
        vk::Format::R8G8_SSCALED == fm ||
        vk::Format::R8G8_UINT == fm ||
        vk::Format::R8G8_SINT == fm ||
        vk::Format::R8G8_SRGB == fm ||
        vk::Format::R16G16_UNORM == fm ||
        vk::Format::R16G16_SNORM == fm ||
        vk::Format::R16G16_USCALED == fm ||
        vk::Format::R16G16_SSCALED == fm ||
        vk::Format::R16G16_UINT == fm ||
        vk::Format::R16G16_SINT == fm ||
        vk::Format::R16G16_SFLOAT == fm ||
        vk::Format::R32G32_UINT == fm ||
        vk::Format::R32G32_SINT == fm ||
        vk::Format::R32G32_SFLOAT == fm 
      ){
        return 2;
      } else if(
        vk::Format::R8G8B8_UNORM == fm ||
        vk::Format::R8G8B8_SNORM == fm ||
        vk::Format::R8G8B8_USCALED == fm ||
        vk::Format::R8G8B8_SSCALED == fm ||
        vk::Format::R8G8B8_UINT == fm ||
        vk::Format::R8G8B8_SINT == fm ||
        vk::Format::R8G8B8_SRGB == fm ||
        vk::Format::B8G8R8_UNORM == fm ||
        vk::Format::B8G8R8_SNORM == fm ||
        vk::Format::B8G8R8_USCALED == fm ||
        vk::Format::B8G8R8_SSCALED == fm ||
        vk::Format::B8G8R8_UINT == fm ||
        vk::Format::B8G8R8_SINT == fm ||
        vk::Format::B8G8R8_SRGB == fm ||
        vk::Format::R16G16B16_UNORM == fm ||
        vk::Format::R16G16B16_SNORM == fm ||
        vk::Format::R16G16B16_USCALED == fm ||
        vk::Format::R16G16B16_SSCALED == fm ||
        vk::Format::R16G16B16_UINT == fm ||
        vk::Format::R16G16B16_SINT == fm ||
        vk::Format::R16G16B16_SFLOAT == fm || 
        vk::Format::R32G32B32_UINT == fm ||
        vk::Format::R32G32B32_SINT == fm ||
        vk::Format::R32G32B32_SFLOAT == fm 
      ){
        return 3;
      } else if(
        vk::Format::R8G8B8A8_UNORM == fm ||
        vk::Format::R8G8B8A8_SNORM == fm ||
        vk::Format::R8G8B8A8_USCALED == fm ||
        vk::Format::R8G8B8A8_SSCALED == fm ||
        vk::Format::R8G8B8A8_UINT == fm ||
        vk::Format::R8G8B8A8_SINT == fm ||
        vk::Format::R8G8B8A8_SRGB == fm ||
        vk::Format::B8G8R8A8_UNORM == fm ||
        vk::Format::B8G8R8A8_SNORM == fm ||
        vk::Format::B8G8R8A8_USCALED == fm ||
        vk::Format::B8G8R8A8_SSCALED == fm ||
        vk::Format::B8G8R8A8_UINT == fm ||
        vk::Format::B8G8R8A8_SINT == fm ||
        vk::Format::B8G8R8A8_SRGB == fm ||
        vk::Format::A8B8G8R8_UNORM_PACK32 == fm ||
        vk::Format::A8B8G8R8_SNORM_PACK32 == fm ||
        vk::Format::A8B8G8R8_USCALED_PACK32 == fm ||
        vk::Format::A8B8G8R8_SSCALED_PACK32 == fm ||
        vk::Format::A8B8G8R8_UINT_PACK32 == fm ||
        vk::Format::A8B8G8R8_SINT_PACK32 == fm ||
        vk::Format::A8B8G8R8_SRGB_PACK32 == fm ||
        vk::Format::A2R10G10B10_UNORM_PACK32 == fm ||
        vk::Format::A2R10G10B10_SNORM_PACK32 == fm ||
        vk::Format::A2R10G10B10_USCALED_PACK32 == fm ||
        vk::Format::A2R10G10B10_SSCALED_PACK32 == fm ||
        vk::Format::A2R10G10B10_UINT_PACK32 == fm ||
        vk::Format::A2R10G10B10_SINT_PACK32 == fm ||
        vk::Format::A2B10G10R10_UNORM_PACK32 == fm ||
        vk::Format::A2B10G10R10_SNORM_PACK32 == fm ||
        vk::Format::A2B10G10R10_USCALED_PACK32 == fm ||
        vk::Format::A2B10G10R10_SSCALED_PACK32 == fm ||
        vk::Format::A2B10G10R10_UINT_PACK32 == fm ||
        vk::Format::A2B10G10R10_SINT_PACK32 == fm ||
        vk::Format::R16G16B16A16_UNORM == fm ||
        vk::Format::R16G16B16A16_SNORM == fm ||
        vk::Format::R16G16B16A16_USCALED == fm ||
        vk::Format::R16G16B16A16_SSCALED == fm ||
        vk::Format::R16G16B16A16_UINT == fm ||
        vk::Format::R16G16B16A16_SINT == fm ||
        vk::Format::R16G16B16A16_SFLOAT == fm ||
        vk::Format::R32G32B32A32_UINT == fm ||
        vk::Format::R32G32B32A32_SINT == fm ||
        vk::Format::R32G32B32A32_SFLOAT == fm
      ){
        return 4;
      } else {
        return 0;
      }
    }
}
