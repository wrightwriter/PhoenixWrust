// ffmpeg::init().unwrap();
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};



use std::io::prelude::*;



use ash::vk;
use image::{ImageBuffer, Rgb};

use crate::res::img::wimage::WImageInfo;
use crate::sys::warenaitems::WAIdxImage;
use crate::wvulkan::WVulkan;

pub struct WVideo {
  pub gpu_image: WAIdxImage,
}
impl WVideo {
  fn monitor (){
  }
  pub fn new(w: &mut WVulkan) -> Self {
    // fn save_file(frame: &Video, index: usize) -> std::result::Result<(), std::io::Error> {
    //     let mut file = File::create(format!("frame{}.ppm", index))?;
    //     file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    //     file.write_all(frame.data(0))?;
    //     Ok(())
    // }
    // vars
    let root_videos_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\videos\\";
    let ffmpeg_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\ffmpeg\\bin\\";
    let ffprobe_path = ffmpeg_dir.clone() + "ffprobe";
    let ffmpeg_path = ffmpeg_dir.clone() + "ffmpeg";

    let video_path = root_videos_dir + "pexels-mart-production-7565438.mp4";
    // let video_path = root_videos_dir + "pexels-vid-2.mp4";

    // READ METADATA
    let duration = {
      let mut cmd = Command::new(&ffprobe_path)
        .args(&[
          "-v",
          "error",
          "-show_entries",
          "format=duration",
          "-of",
          "default=noprint_wrappers=1:nokey=1",
          &video_path,
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

      let stdout = cmd.stdout.as_mut().unwrap();
      let stdout_reader = BufReader::new(stdout);

      let duration = stdout_reader.lines().next().unwrap().unwrap();

      let w = cmd.wait().unwrap();
      println!("{}", duration);

      duration.parse::<f64>().unwrap() - 1.0
    };

    let dimensions = {
      let mut cmd = Command::new(&ffprobe_path)
        .args(&[
          "-v",
          "error",
          "-show_entries",
          "stream=width,height",
          "-of",
          "csv=p=0:s=x",
          &video_path,
        ])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

      let stdout = cmd.stdout.as_mut().unwrap();
      let stdout_reader = BufReader::new(stdout);

      let dimensions = stdout_reader
        .lines()
        .next()
        .unwrap()
        .unwrap()
        .split('x')
        .map(|dim| dim.parse::<u32>().unwrap())
        .collect::<Vec<_>>();

      let w = cmd.wait().unwrap();
      println!("{}", dimensions[0]);
      println!("{}", dimensions[1]);

      dimensions
    };
    let aspect_preserved_width = dimensions[1] * 3;
    let barcode_dimensions = vec![aspect_preserved_width, dimensions[1]];

    // RIP

    pub type Pixel = Rgb<u8>;
    pub type FrameBuffer = ImageBuffer<Pixel, Vec<u8>>;

    // let pixels = {

    let fps_dividend = duration / dimensions[0] as f64;

    let mut cmd = Command::new("ffmpeg");
    cmd.args(&[
      "-i",
      &video_path,
      "-f",
      "image2pipe",
      "-vcodec",
      "rawvideo",
      "-pix_fmt",
      "rgb24",
      // "-vf",
      // &format!("fps=1/{:?}", fps_dividend),

      // "-r",
      // "15",
      // "-movflags",
      // "+faststart",
      "-",
    ]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    let mut child = cmd.spawn().unwrap();

    let child_stdout = &mut child.stdout.take().unwrap();

    let mut pixels = vec![0; (dimensions[0] * dimensions[1] * 3) as usize];
    // unsafe{
    //     pixels.set_len(pixels.capacity());
    // }

    for pixel in &mut pixels {
      *pixel = 0u8;
    }

    // image::save
    let mut img_idx = 0;
    let vec_len = pixels.len();

    let mut buff = [0u8; 100_000 as usize];
    loop {
      let bytes_read_cnt = child_stdout.read(&mut buff).unwrap();

      // let bytes_read_cnt = child_stdout.read;

      //     let buff_reader = BufReader::new(stdout);
      //     let buff = buff_reader.buffer();

      println!("{}", bytes_read_cnt);
      println!("{}", pixels.len());

      let mut br = false;
      for i in 0..bytes_read_cnt {
        pixels[img_idx] = buff[i];

        img_idx += 1;
        if img_idx >= vec_len {
          img_idx = 0;
          br = true;
        }
        // println!("{}",*pixel);
      }
      if br {
        break;
      }
    }
    // image::save_buffer_with_format(
    //   "./TEST.png",
    //   &pixels,
    //   dimensions[0],
    //   dimensions[1],
    //   image::ColorType::Rgb8,
    //   image::ImageFormat::Png,
    // );

    // println!("{}", "potato");
    // println!("{}", "potato");

    let vk_format = vk::Format::R8G8B8_UNORM;

    let create_info = WImageInfo {
      // UB!!!!!!!!!
      resx: dimensions[0],
      resy: dimensions[1],
      raw_pixels: Some(pixels.as_mut_ptr()),
      format: vk::Format::R8G8B8_UNORM,
      ..wdef!()
    };

    let gpu_image = w.w_tl.new_image(&mut w.w_device, create_info).0;
    
    

    Self { gpu_image }
  }
}
