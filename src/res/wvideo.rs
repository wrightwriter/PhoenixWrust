use std::borrow::Borrow;
// ffmpeg::init().unwrap();
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};



use std::io::prelude::*;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

extern crate ffmpeg_next as ffmpeg;


use ash::vk;
use ffmpeg::Discard;
use fragile::Fragile;
use image::{ImageBuffer, Rgb};

use crate::res::buff::wbuffer::WBuffer;
use crate::res::img::wimage::WImageInfo;
use crate::sys::warenaitems::{WAIdxImage, WArenaItem};
use crate::wvulkan::WVulkan;

pub struct WVideo {
  pub gpu_image: WAIdxImage,
  pub staging_buff: WBuffer,
  pub dimensions: [u32; 2],
}

impl WVideo {
  fn monitor (){
  }

  #[profiling::function]
  pub fn new(w: &mut WVulkan) -> Self {
    use ffmpeg::format::{input, Pixel};
    use ffmpeg::media::Type;
    use ffmpeg::software::scaling::{context::Context, flag::Flags};
    use ffmpeg::util::frame::video::Video;
    // fn save_file(frame: &Video, index: usize) -> std::result::Result<(), std::io::Error> {
    //     let mut file = File::create(format!("frame{}.ppm", index))?;
    //     file.write_all(format!("P6\n{} {}\n255\n", frame.width(), frame.height()).as_bytes())?;
    //     file.write_all(frame.data(0))?;
    //     Ok(())
    // }

    ffmpeg::init().unwrap();


    // vars
    let root_videos_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\src\\videos\\";
    // let ffmpeg_dir = std::env::var("WORKSPACE_DIR").unwrap() + "\\ffmpeg\\bin\\";
    // let ffprobe_path = ffmpeg_dir.clone() + "ffprobe";
    // let ffmpeg_path = ffmpeg_dir.clone() + "ffmpeg";

    let video_path = root_videos_dir + "pexels-mart-production-7565438.mp4";
    // let video_path = root_videos_dir + "pexels-vid-2.mp4";
    let video_path = video_path.as_str();
    // let video_path = root_videos_dir + "pexels-vid-2.mp4";
    
    let mut context = input(&video_path).unwrap();

    for (k, v) in context.metadata().iter() {
        println!("{}: {}", k, v);
    }

    if let Some(stream) = context.streams().best(ffmpeg::media::Type::Video) {
        println!("Best video stream index: {}", stream.index());
    }
    let duration = context.duration() as f64 / f64::from(ffmpeg::ffi::AV_TIME_BASE);
    
    println!( "duration (seconds): {:.2}", duration);


    let input_stream = context
        .streams()
        .best(Type::Video)
        .ok_or(ffmpeg::Error::StreamNotFound).unwrap();

    let video_stream_index = input_stream.index();

    

    // let time_base = input_stream.time_base();
    // let time_num = time_base.numerator();
    // let time_den = time_base.denominator();
    
    // let time_ratio = (time_num as f64)/(time_den as f64);

    // println!("--------- VIDEO ---------");
    // println!("{}", time_num);
    // println!("{}", time_den);
    // println!("--------- VIDEO ---------");
    // panic!();

    let context_decoder = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters()).unwrap();
    



    let mut decoder = context_decoder.decoder().video().unwrap();

    // decoder.skip_loop_filter(Discard::None);


    let mut dimensions = [
        decoder.width().clone(),
        decoder.height().clone(),
    ];
    let dimensions_clone = dimensions.clone();

    let (chan_thread_init_finished_sender, chan_thread_init_finished_receiver) = channel();
    let (chan_main_init_finished_sender, chan_main_init_finished_receiver) = channel();

    std::thread::spawn(move ||{
      // profiling::tracy_client::th
      profiling::register_thread!();
      profiling::tracy_client::set_thread_name!("VIDEO THREAD");

      profiling::scope!("video outer");


      let mut dims = dimensions_clone;

      let mut rgb_frame = Video::empty();
      // dims[0] = decoder.width();
      // dims[1] = decoder.height();

      // init_clone.clone().send(());
      chan_thread_init_finished_sender.send(());

      let (sz_bytes, stag_buff_mapped_mem): (u32, usize) = chan_main_init_finished_receiver
        .recv()
        .expect("Error: timed out.");

      let sz_bytes = dims[0] * dims[1] * 4 as u32;

      println!("--------- EPIC THREAD ---------");
      println!("{}", dims[0]);
      println!("{}", dims[1]);
      println!("--------- EPIC THREAD ---------");
      
      
      let stag_buff_mapped_mem = stag_buff_mapped_mem as *mut u8;


      let mut scaler = Context::get(
          decoder.format(),
          decoder.width(),
          decoder.height(),
          // Pixel::RGB24,
          Pixel::RGBA,
          decoder.width(),
          decoder.height(),
          Flags::BITEXACT ,
      ).unwrap();
      
      let mut frame_index = 0;


      let mut decoded = Video::empty();

      unsafe{
        loop{
          context.seek(0, std::ops::Range{start: 0, end: 5 });
          let mut fr = 0;

          let t_start = SystemTime::now();
          for (stream, packet) in context.packets() {
              if stream.index() == video_stream_index {

                  let time_base = stream.time_base();
                  let time_num = time_base.numerator();
                  let time_den = time_base.denominator();
                  
                  let time_ratio = (time_num as f64)/(time_den as f64);

                  // println!("--------- VIDEO ---------");
                  // println!("{}", time_den);
                  // panic!();
                  let rate = stream.rate().0 as f64;
                  decoder.send_packet(&packet).unwrap();

                  // println!("--------- VIDEO ---------");
                  // println!("{}", stream.rate().0);
                  // println!("{}", stream.rate().1);
                  // println!("--------- VIDEO ---------");

                  // decoder.set_parameters(ffmpeg::codec::Parameters::)
                  // let dec_res = decoder.receive_frame(&mut decoded);
                  while decoder.receive_frame(&mut decoded).is_ok() {

                      profiling::scope!("video frame");
                      scaler.run(&decoded, &mut rgb_frame).unwrap();
                      let pts = decoded.pts().unwrap() as f64;
                      
                      let t_desired = pts * time_ratio;

                      let time_now = SystemTime::now();
                      let dur_since_start = time_now.duration_since(t_start).unwrap();

                      let dur_delay = dur_since_start.as_secs_f64() - t_desired;

                      if dur_delay.is_sign_positive(){

                        profiling::scope!("video sleep");
                        std::thread::sleep(Duration::from_secs_f64(dur_delay/rate*10.));
                      }
                      
                      std::ptr::copy_nonoverlapping(rgb_frame.data(0).as_ptr(), stag_buff_mapped_mem, sz_bytes as usize);

                      frame_index += 1;
                      fr += 1;
                      // if frame_index % 400 == 0{
                      // if fr == 400{
                      //   println!("{}", frame_index);
                      //   break;
                      // }
                  }
              }
          }
        }
      }

    });

    
    chan_thread_init_finished_receiver 
      .recv()
      .expect("Error: timed out.");



    let create_info = WImageInfo {
      resx: dimensions[0],
      resy: dimensions[1],
      format: vk::Format::R8G8B8A8_UNORM,
      ..wdef!()
    };

    let gpu_image = w.w_tl.new_image(&mut w.w_device, create_info.clone()).0;

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

    chan_main_init_finished_sender.send((
        sz_bytes, staging_buff.mapped_mems[0] as usize
    ));

    
    std::thread::sleep(Duration::from_secs(2));

    let mut s = Self { gpu_image, dimensions, staging_buff };
    let cmd_buff = s.update_frame(w);
    

    unsafe{
      w.w_device.device.queue_submit(
        w.w_device.queue, 
        // submits, 
        &[vk::SubmitInfo::builder().command_buffers(&[cmd_buff]).build()],
        vk::Fence::null()
        // fence
      );
    }
    
    s
  }
  pub fn update_frame(
    &mut self,
    w: &mut WVulkan
  ) -> vk::CommandBuffer{
    let cmd_buff = &w.w_device.curr_pool().get_cmd_buff();
    unsafe{
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
        }).build();
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
