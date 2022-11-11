use std::{fs, process::{Stdio, Command}, ops::Add, ffi::OsStr};

use imgui::Condition;
use stb_image::stb_image::bindgen::stbi_set_flip_vertically_on_load;

use crate::{
  abs::wcam::WCamera,
  res::{buff::wwritablebuffertrait::UniformEnum, img::wimage::WImage},
  wvulkan::WVulkan,
};

use super::{
  warenaitems::{WAIdxImage, WArenaItem},
  wdevice::{WDevice, GLOBALS},
  wmanagers::WTechLead,
  wshaderman::WShaderMan,
  wtime::WTime,
};

pub struct WRecorder {
  pub recording: bool,
  pub frame_rate: u32,
  pub frame_cnt: u32,
  pub length: f32,
  frame: u32,

  pub pixels: Vec<u8>,

  video_dir: String,
  video_frames_dir: String,
}

impl WRecorder {
  pub fn new() -> Self {
    let video_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\recs\\";
    let video_frames_dir = video_dir.clone() + "frames\\";

    fs::create_dir_all(&video_dir).unwrap();
    fs::create_dir_all(&video_frames_dir).unwrap();

    Self {
      recording: false,
      frame_rate: 30,
      frame_cnt: 400,
      frame: 0,
      video_dir,
      video_frames_dir,
      pixels: vec![],
      length: 2.0,
    }
  }

  pub fn start_recording(
    &mut self,
    w_cam: &mut WCamera,
    w_time: &mut WTime,
    frame_rate: u32,
    // length: f32,
  ) {
    self.recording = true;
    self.frame_rate = frame_rate;
    self.frame_cnt = (self.length * (frame_rate as f32)) as u32;
    self.frame = 0;

    w_time.reset();
    
    for path in fs::read_dir(&self.video_frames_dir).unwrap(){
      let path = path.unwrap().path();
      let extension = path.extension().unwrap();
      if extension == OsStr::new("png") || extension == OsStr::new("bmp"){
        fs::remove_file(path).unwrap();
      }
    }
    

    unsafe {
      self.pixels.set_len(0);
      self.pixels.reserve((w_cam.width * w_cam.width * 4) as usize);
    }
  }

  pub fn try_recording(
    &mut self,
    img: &WImage,
    w_device: &mut WDevice,
  ) {
    if !self.recording {
      return;
    } 
    if self.frame >= self.frame_cnt{
      self.end_recording();
      return;
    }

    if self.frame != 0{
      WTechLead::copy_swapchain_to_cpu_image(w_device, img, &mut self.pixels);

      image::save_buffer_with_format(
        self.video_frames_dir.clone() + &self.frame.to_string() + ".bmp",
        &self.pixels,
        img.resx,
        img.resy,
        image::ColorType::Rgba8,
        image::ImageFormat::Bmp,
      )
      .unwrap();
    }

    self.frame += 1;
  }

  pub fn end_recording(&mut self) {
    self.recording = false;
    self.frame = 0;

    // var command: String = "ffmpeg "
    // command += "-framerate " + frameRate.toInt() + " "
    // command +=  "-i " + dirPath + "\\%d.png "
    // command += "-c:v libx264 -pix_fmt yuv420p "
    // command +=  "-crf " + sketch.recordingSettings.crf + " "
    // command += dirPath + "\\_OUT.mp4"

    let mut cmd = Command::new("ffmpeg")
      .args(&[
        "-framerate",
        &self.frame_rate.to_string(),
        "-y",
        "-i",
        &self.video_frames_dir.clone().add("\\%d.bmp"),

        "-c:v",
        "libx264",

        "-pix_fmt",
        "yuv420p",

        "-crf",
        "15",
        &self.video_dir.clone().add(&"OUT.mp4"),
      ])
      .stdout(Stdio::piped())
      .spawn()
      .unwrap();

    cmd.wait().unwrap();
  }

  fn composite_frames() {}

}
