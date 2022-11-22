use std::{
  fs,
  process::{Command, Stdio},
};

use ash::vk;

use crate::{
  res::img::wimage::WImageInfo,
  sys::{warenaitems::{WAIdxBuffer, WAIdxImage}, wtl::WTechLead},
  wvulkan::WVulkan,
};

#[derive(Default, Copy, Clone)]
struct WGlyph {
  pub unicode: u32,
  pub advance: f32,

  pub plane_bound_left: f32,
  pub plane_bound_bot: f32,
  pub plane_bound_right: f32,
  pub plane_bound_top: f32,

  pub atlas_bound_left: f32,
  pub atlas_bound_bot: f32,
  pub atlas_bound_right: f32,
  pub atlas_bound_top: f32,
}

// #[derive(Debug)]
type WGlyphArray = [WGlyph; 255];

#[derive(Clone,Copy)]
pub struct WFont {
  glyphs: WGlyphArray,
  pub gpu_metadata_buff: WAIdxBuffer,
  pub gpu_atlas: WAIdxImage,
}
impl WFont {
  pub fn new<S: Into<String>>(
    w: &mut WVulkan,
    w_tl: &mut WTechLead,
    font_path: S,
  ) -> Self {
    let (glyphs, atlas_file_path) = Self::load_from_msdf(font_path.into());
    // let w_tl = &mut w.w_tl;

    let gpu_metadata_buff = {
      let buff = w_tl.new_buffer(
        w,
        vk::BufferUsageFlags::STORAGE_BUFFER,
        (glyphs.len()*std::mem::size_of::<WGlyph>()) as u32,
        false,
      );

      use std::ptr::copy_nonoverlapping as memcpy;
      unsafe {
        let _gpu_metadata_buff = buff.1;
        _gpu_metadata_buff.map(&w.w_device.device);
        memcpy::<WGlyph>(glyphs.as_ptr(), _gpu_metadata_buff.mapped_mems[0].cast(), glyphs.len());
      }

      buff.0
    };

    let gpu_atlas = w_tl.new_image(
      w,
      WImageInfo {
        file_path: Some(atlas_file_path),
        ..wdef!()
      },
    );

    Self {
      glyphs,
      gpu_metadata_buff: gpu_metadata_buff,
      gpu_atlas: gpu_atlas.0,
    }
  }
  fn load_from_msdf(font_path: String) -> (WGlyphArray, String) {
    let root_fonts_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\msdf\\";

    let font_name: String = "ferritecore.otf".to_owned();

    let msdf_atlas_gen_path = root_fonts_dir.clone() + "msdf-atlas-gen.exe";

    let font_path = root_fonts_dir.clone() + &font_name;

    let charset_path = root_fonts_dir.clone() + "charset.txt";

    let out_image_path = &font_path.replace("otf", "png");

    let out_json_path = &font_path.replace("otf", "json");

    let mut cmd = Command::new(&msdf_atlas_gen_path)
      .args(&[
        "-font",
        &font_path,
        // "-charset",
        // &charset_path,
        "-imageout",
        &out_image_path,
        "-json",
        &out_json_path,
        "-type",
        "msdf",
      ])
      .stdout(Stdio::piped())
      .spawn()
      .unwrap();

    cmd.wait();

    // Parse json
    let json_string = fs::read_to_string(&out_json_path).unwrap();

    let json: serde_json::Value = serde_json::from_str(&json_string).unwrap();

    let json_glyphs = json.get("glyphs").unwrap().as_array().unwrap();

    let mut w_glyphs: WGlyphArray = [WGlyph::default(); 255];

    for glyph in json_glyphs {
      let plane_bounds = glyph.get("planeBounds");
      let atlas_bounds = glyph.get("atlasBounds");
      if let Some(plane_bounds) = plane_bounds {
        if let Some(atlas_bounds) = atlas_bounds {
          let w_glyph = WGlyph {
            unicode: glyph.get("unicode").unwrap().as_u64().unwrap() as u32,
            advance: glyph.get("advance").unwrap().as_f64().unwrap() as f32,

            plane_bound_left: plane_bounds.get("left").unwrap().as_f64().unwrap() as f32,
            plane_bound_bot: plane_bounds.get("bottom").unwrap().as_f64().unwrap() as f32,
            plane_bound_right: plane_bounds.get("right").unwrap().as_f64().unwrap() as f32,
            plane_bound_top: plane_bounds.get("top").unwrap().as_f64().unwrap() as f32,

            atlas_bound_left: atlas_bounds.get("left").unwrap().as_f64().unwrap() as f32,
            atlas_bound_bot: atlas_bounds.get("bottom").unwrap().as_f64().unwrap() as f32,
            atlas_bound_right: atlas_bounds.get("right").unwrap().as_f64().unwrap() as f32,
            atlas_bound_top: atlas_bounds.get("top").unwrap().as_f64().unwrap() as f32,
          };

          w_glyphs[w_glyph.unicode as usize] = w_glyph;
        }
      }
    }
    // for glyph in w_glyphs {
    //   println!("{}", glyph.atlas_bound_left);
    //   println!("{}", glyph.atlas_bound_bot);
    //   println!("{}", glyph.atlas_bound_right);
    //   println!("{}", glyph.atlas_bound_top);
    // }
    // println!("{}", "potato");

    (w_glyphs, out_image_path.clone())
  }
}
