use std::borrow::Borrow;
// ffmpeg::init().unwrap();
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

use std::io::prelude::*;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

extern crate ffmpeg_next as ffmpeg;

use ash::vk;
use ffmpeg::ffi::{av_seek_frame, AVFormatContext};
use ffmpeg::Discard;
use fragile::Fragile;
use image::{ImageBuffer, Rgb};

use crate::res::buff::wbuffer::WBuffer;
use crate::res::img::wimage::WImageInfo;
use crate::sys::warenaitems::{WAIdxImage, WArenaItem};
use crate::sys::wtl::WTechLead;
use crate::wvulkan::WVulkan;

use ffmpeg::format::{input, Pixel};
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context, flag::Flags};
use ffmpeg::util::frame::video::Video;

pub struct WVideo {
  pub gpu_image: WAIdxImage,
  pub staging_buff: WBuffer,
  pub dimensions: [u32; 2],
  // pub chan_seek_sender: Sender<f32>,
  pub seek: Arc<Mutex<f32>>,
  pub speed: Arc<Mutex<f64>>,
}

impl WVideo {
  pub fn set_speed(&mut self, speed: f64){
    *self.speed.lock().unwrap() = speed;
  }
  #[profiling::function]
  pub fn new<S: Into<String>>(
    w: &mut WVulkan,
    w_tl: &mut WTechLead,
    _path: S,
  ) -> Self {
    // fn save_file(frame: &Video, index: usize) -> std::result::Result<(), std::io::Error> {
    //     let mut file = File::create(format!("frame{}.ppm", index))?;
    //     file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    //     file.write_all(frame.data(0))?;
    //     Ok(())
    // }

    // vars
    let root_videos_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\videos\\";

    let video_path = root_videos_dir + &_path.into();
    let video_path = video_path.as_str();

    let (chan_thread_init_finished_sender, chan_thread_init_finished_receiver) = channel();
    let (chan_main_init_finished_sender, chan_main_init_finished_receiver) = channel();

    let seek = Arc::new(Mutex::new(-1.0f32));
    let speed = Arc::new(Mutex::new(1.0f64));
    let seek_clone = seek.clone();
    let speed_clone = speed.clone();

    let mut context = input(&video_path).unwrap();

    for (k, v) in context.metadata().iter() {
      println!("{}: {}", k, v);
    }

    if let Some(stream) = context.streams().best(ffmpeg::media::Type::Video) {
      println!("Best video stream index: {}", stream.index());
    }
    let duration = context.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE);

    println!("duration (seconds): {:.2}", duration);

    let input_stream = context.streams().best(Type::Video).ok_or(ffmpeg::Error::StreamNotFound).unwrap();

    let video_stream_index = input_stream.index();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters()).unwrap();

    let mut decoder = context_decoder.decoder().video().unwrap();

    let dimensions = [decoder.width().clone(), decoder.height().clone()];

    wprint!("VIDEO DIM");
    wprint!(dimensions[0]);
    wprint!(dimensions[1]);

    let time_base = input_stream.time_base().clone();
    let time_num = time_base.numerator();
    let time_den = time_base.denominator();

    let time_ratio = (time_num as f64) / (time_den as f64);

    std::thread::spawn(move || {
      // profiling::tracy_client::th
      profiling::register_thread!();
      profiling::tracy_client::set_thread_name!("VIDEO THREAD");

      profiling::scope!("video outer");

      let mut dims = dimensions.clone();

      chan_thread_init_finished_sender.send(dims);

      let mut rgb_frame = Video::empty();
      let mut decoded = Video::empty();

      let (sz_bytes, stag_buff_mapped_mem): (u32, usize) = chan_main_init_finished_receiver.recv().expect("Error: timed out.");
      let sz_bytes = dims[0] * dims[1] * 4 as u32;
      let stag_buff_mapped_mem = stag_buff_mapped_mem as *mut u8;

      let mut scaler = Context::get(
        decoder.format(),
        // decoder.width(),
        // decoder.height(),
        dims[0],
        dims[1],
        // Pixel::RGB24,
        // Pixel::RGBA,
        Pixel::RGBA,
        // decoder.width(),
        // decoder.height(),
        dims[0],
        dims[1],
        // Flags::BITEXACT ,
        Flags::LANCZOS,
      )
      .unwrap();

      let mut frame_index = 0;

      unsafe {
        let mut seek_to = 0.0f32;
        loop {
          context.seek(
            (seek_to as f64 / time_ratio) as i64,
            std::ops::Range {
              start: i64::min_value(),
              end: i64::max_value(),
            },
          );

          wprint!("VIDEO STREAM BEGIN");

          // context.play()

          seek_to = 0.0f32;

          let mut fr = 0;

          let mut ts_idx = 0.0;
          let mut ts_prev = 0.0;
          let mut delta_ts = 0.0;

          let t_start = SystemTime::now();
          let mut t_prev = SystemTime::now();

          let mut pack_idx = 0;

          for (stream, mut packet) in context.packets() {
            if stream.index() == video_stream_index {
              // decoder.skip_frame(Discard::All);
              // decoder.send_packet(&packet).unwrap();

              // while decoder.receive_frame(&mut decoded).is_ok() {
              // }

              // if pack_idx % 2 != 1{
              // //   // continue;

              //   //  .skip_frame(Discard::All);

              //   // while decoder.receive_frame(&mut decoded).is_ok() {
              //   // }
              // } else
              {
                decoder.skip_frame(Discard::Default);
                // packet.set_position(0);
                // let rate = stream.rate().0 as f64;
                decoder.send_packet(&packet).unwrap();
                // decoder.has_b_frames();
                // println!("POTATO");
                // println!("{}", packet.dts().unwrap());
                // println!("{}", packet.pts().unwrap());
                // println!("{}", t_desired);
                // ------------ single frame
                let mut ff = false;
                while decoder.receive_frame(&mut decoded).is_ok() {
                  profiling::scope!("video transfer");
                  let timestamp = decoded.timestamp().unwrap() as f64;
                  if fr == 0 {
                    ts_idx = timestamp;
                  } else {
                    delta_ts = timestamp - ts_prev;
                    ts_idx += delta_ts;
                  };

                  let sp = *speed_clone.lock().unwrap();

                  scaler.run(&decoded, &mut rgb_frame).unwrap();
                  let pts = decoded.pts().unwrap() as f64;
                  // let dts = decoded.ts;

                  // println!("ts: {}", timestamp);
                  // println!("pts: {}", pts);
                  // println!("dts: {}", dts);

                  // if(fr % 10 == 0){
                  // }


                  // let dt_desired_secs = if sp >= 1.0{
                  //   delta_ts * time_ratio * sp
                  // } else {
                  //   -delta_ts * time_ratio * ( (sp - 1.0) * -1.0)
                  // };
                  let dt_desired_secs = delta_ts * time_ratio * sp;
                  


                  let mut time_now = SystemTime::now();
                  let mut dt_secs = time_now.duration_since(t_prev).unwrap().as_secs_f64();
                  let mut delay_secs = dt_desired_secs - dt_secs;

                  // if fr % 10 == 0{

                  //   wprint!("STATS");
                  //   wprint!(dt_desired_secs);
                  //   wprint!(sp);
                  //   wprint!(delay_secs);
                  // }

                  // dur_delay = 0f64;

                  // let dur_since_start = time_now.duration_since(t_start).unwrap();
                  // let mut dur_delay = t_desired - dur_since_start.as_secs_f64();
                  std::ptr::copy_nonoverlapping(rgb_frame.data(0).as_ptr(), stag_buff_mapped_mem, sz_bytes as usize);

                  if delay_secs > 1.0 {
                    delay_secs = 0.;
                  }

                  if delay_secs >= 0.0 {
                    profiling::scope!("video sleep");
                    let wait_dur = Duration::from_secs_f64(delay_secs);
                    std::thread::sleep(wait_dur);
                    // std::thread::sleep(Duration::from_secs_f64(0.00001));
                    // println!("delay: {}", dur_delay);
                    time_now += wait_dur;
                  } 
                  // if timestamp * time_ratio > 1.0{


                  // }
                  // else if delay_secs < -0.05 && fr > 15 {
                    
                  //   // TODO: seek forward
                  //   // let mut s = seek_clone.lock().unwrap();
                  //   // if (*s).is_sign_positive() {
                  //   wprint!("FAST FORWARD");
                  //   // seek_to = (timestamp * time_ratio + delay_secs) as f32;
                  //   seek_to = (ts_idx * time_ratio - delay_secs) as f32;
                  //   // *s = -1.0f32;
                  //   ff = true;
                  //   break;
                  // }


                  frame_index += 1;
                  fr += 1;

                  t_prev = time_now;
                  ts_prev = timestamp;
                  // println!(
                  //   "fpss: {}",
                  //   (fr as f64)/time_now.duration_since(t_start).unwrap().as_secs_f64()
                  // );
                }

                if ff {
                  break;
                }
                let mut s = seek_clone.lock().unwrap();
                if (*s).is_sign_positive() {
                  seek_to = *s;
                  *s = -1.0f32;
                  break;
                }
              }
            }
            // wprint!("END PACKET");
          }
        }
      }
    });

    let r = chan_thread_init_finished_receiver.recv().expect("Error: timed out.");

    let mut dimensions = r;

    let create_info = WImageInfo {
      resx: dimensions[0],
      resy: dimensions[1],
      format: vk::Format::R8G8B8A8_UNORM,
      ..wdef!()
    };

    let gpu_image = w_tl.new_image(w, create_info.clone()).0;

    let mut sz_bytes = 0;
    sz_bytes = create_info.resx * create_info.resy * 4 as u32;

    // let staging_buff = w.w_tl.new_buffer(w, usage, sz_bytes, pongable)

    let mut staging_buff = WBuffer::new(
      &w.w_device.device,
      &mut w.w_device.allocator,
      vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::TRANSFER_SRC,
      sz_bytes as u32,
      false,
    );
    staging_buff.map(&w.w_device.device);

    chan_main_init_finished_sender.send((sz_bytes, staging_buff.mapped_mems[0] as usize));

    std::thread::sleep(Duration::from_secs(2));

    let mut s = Self {
      gpu_image,
      dimensions,
      staging_buff,
      seek,
      speed,
    };
    let cmd_buff = s.update_frame(w);

    unsafe {
      w.w_device.device.queue_submit(
        w.w_device.queue,
        // submits,
        &[vk::SubmitInfo::builder().command_buffers(&[cmd_buff]).build()],
        vk::Fence::null(), // fence
      );
    }

    s
  }

  pub fn seek(
    &mut self,
    t: f32,
  ) {
    let mut s = self.seek.lock().unwrap();
    *s = t;
  }
  pub fn update_frame(
    &mut self,
    w: &mut WVulkan,
  ) -> vk::CommandBuffer {
    let cmd_buff = &w.w_device.curr_pool().get_cmd_buff();
    unsafe {
      let cmd_buf_begin_info = vk::CommandBufferBeginInfo::builder().build();
      let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1)
        .build();

      let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D {
          width: self.dimensions[0],
          height: self.dimensions[1],
          depth: 1,
        })
        .build();
      w.w_device.device.begin_command_buffer(*cmd_buff, &cmd_buf_begin_info);

      let img_borrow = self.gpu_image.get();
      w.w_device.device.cmd_copy_buffer_to_image(
        *cmd_buff,
        self.staging_buff.get_handle(),
        img_borrow.handle,
        vk::ImageLayout::GENERAL,
        &[region],
      );
      w.w_device.device.end_command_buffer(*cmd_buff);

      *cmd_buff
      // w.w_device.device.queue_submit(
      //   w.w_device.queue,
      //   // submits,
      //   &[vk::SubmitInfo::builder().command_buffers(&[*cmd_buff]).build()],
      //   vk::Fence::null()
      //   // fence
      // );
    }
  }

  // let p = ffmpeg::ffi::AVCodecParameters{
  //     codec_type: todo!(),
  //     codec_id: todo!(),
  //     codec_tag: todo!(),
  //     extradata: todo!(),
  //     extradata_size: todo!(),
  //     format: todo!(),
  //     bit_rate: todo!(),
  //     bits_per_coded_sample: todo!(),
  //     bits_per_raw_sample: todo!(),
  //     profile: todo!(),
  //     level: todo!(),
  //     width: todo!(),
  //     height: todo!(),
  //     sample_aspect_ratio: todo!(),
  //     field_order: todo!(),
  //     color_range: todo!(),
  //     color_primaries: todo!(),
  //     color_trc: todo!(),
  //     color_space: todo!(),
  //     chroma_location: todo!(),
  //     video_delay: todo!(),
  //     channel_layout: todo!(),
  //     channels: todo!(),
  //     sample_rate: todo!(),
  //     block_align: todo!(),
  //     frame_size: todo!(),
  //     initial_padding: todo!(),
  //     trailing_padding: todo!(),
  //     seek_preroll: todo!(),
  //   ..wdef!()
  // };

  //   Self { gpu_image }
}
