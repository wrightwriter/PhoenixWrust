use imgui::{Condition, SliderFlags};

use crate::{abs::wcam::WCamera, res::buff::wwritablebuffertrait::UniformEnum};

use super::{
  warenaitems::WArenaItem,
  wdevice::{WDevice, GLOBALS},
  wshaderman::WShaderMan,
  wtime::WTime, wrecorder::WRecorder,
};

pub struct WGUI {}

impl WGUI {
  pub fn new() -> Self {
    Self {}
  }

  pub fn draw_internal(
    &self,
    // WV: &mut WVulkan,
    w_device: &mut WDevice,
    w_time: &mut WTime,
    im_ui: &mut imgui::Ui,
    w_shader_man: &mut WShaderMan,
    w_cam: &mut WCamera,
    w_recorder: &mut WRecorder,
    gui_enabled: bool
  ) {
    // Shader errors
    {
      let shaders_with_errors = &mut *w_shader_man.shaders_with_errors.lock().unwrap();
      if shaders_with_errors.len() > 0 {
        let im_w = imgui::Window::new("b")
          .position([10.0, 10.0], Condition::Always)
          .collapsed(true, Condition::Always)
          .flags(
            imgui::WindowFlags::NO_TITLE_BAR
              .union(imgui::WindowFlags::ALWAYS_AUTO_RESIZE)
              .union(imgui::WindowFlags::NO_RESIZE)
              // .union(imgui::WindowFlags::)
              .union(imgui::WindowFlags::NO_MOVE)
          )
          // .content_size([w_cam.width as f32 - 20., 0.0]);
          .size([w_cam.width as f32 - 20., 0.0], Condition::Always);
        
          // .size([w_cam.width as f32 - 20., (w_cam.height / 3) as f32], Condition::Always);

        im_w.build(&im_ui, || {
          let mut col: [f32; 3] = [1., 0., 0.];
          imgui::ColorEdit::new(" ", &mut col)
            .flags(imgui::ColorEditFlags::NO_INPUTS.union(imgui::ColorEditFlags::NO_PICKER))
            .build(&im_ui);
          // im_ui.text_wrapped(
          //   " ----  SHADER ERROR: "
          //   .to_string(),
          // );

          for prog in shaders_with_errors {
            let p = prog.get();
            if let Some(sh) = &p.frag_shader{
                im_ui.text_wrapped(">>>>> ".to_string() + &sh.file_name);
                im_ui.text_wrapped(&sh.compilation_error);
            }
            if let Some(sh) = &p.comp_shader{
                im_ui.text_wrapped(">>>>> ".to_string() + &sh.file_name);
                im_ui.text_wrapped(&sh.compilation_error);
            }
            if let Some(sh) = &p.vert_shader{
                im_ui.text_wrapped(">>>>> ".to_string() + &sh.file_name);
                im_ui.text_wrapped(&sh.compilation_error);
            }
          }
        });
      }
    }

    if !gui_enabled {
      return;
    }

    // FPS
    {
      let im_w = imgui::Window::new("a ")
        .position([10.0, 10.0], Condition::Always)
        .collapsed(true, Condition::Always)
        .flags(
          imgui::WindowFlags::NO_TITLE_BAR
            .union(imgui::WindowFlags::ALWAYS_AUTO_RESIZE)
            .union(imgui::WindowFlags::NO_RESIZE)
            .union(imgui::WindowFlags::NO_MOVE)
            .union(imgui::WindowFlags::NO_TITLE_BAR),
        )
        .size([700.0, 500.0], Condition::Always)
        .draw_background(false);

      im_w.build(&im_ui, || {
        im_ui.text("s: ".to_string() + &w_time.dt_f64.to_string());
        im_ui.text("fps: ".to_string() + &(w_time.fps as u32).to_string());
      });
    }

    // Exposed uniforms
    // Recording
    unsafe {
      let ubos = &mut (*GLOBALS.shared_ubo_arena);
      let im_w = imgui::Window::new("Settings");

      let mut a: Vec<i32> = vec![];
      let b = a.insert(0, 3);

      im_w.build(&im_ui, || {
        // if im_ui.button("Recording"){
        imgui::Drag::new("Rec length").build(&im_ui, &mut w_recorder.length);

        imgui::Drag::new("Rec fps")
          .speed(0.1f32).display_format("%d").build(&im_ui, &mut w_recorder.frame_rate);

        if im_ui.button("Recording"){
          w_recorder.start_recording(w_cam, w_time, w_recorder.frame_rate);
        }

        for ubo in ubos {
          let mut i = 0;
          for name in &ubo.1.uniforms.uniforms_names {
            let val = &mut ubo.1.uniforms.uniforms[i];
            match val {
              UniformEnum::F32(__) => {
                // im_ui.input_float(name, __);
                imgui::Drag::new(name).build(&im_ui, __);
                // imgui::InputFloat::new(&im_ui, name, __)
                // .build();
              }
              UniformEnum::F64(__) => {}
              UniformEnum::U64(__) => {}
              UniformEnum::U32(__) => {}
              UniformEnum::U16(__) => {}
              UniformEnum::U8(__) => {}
              UniformEnum::VEC2(__) => {}
              UniformEnum::VEC3(__) => {}
              UniformEnum::VEC4(__) => {}
              UniformEnum::MAT4X4(__) => {}
              UniformEnum::ARENAIDX(__) => {}
            }
            i += 1;
          }
        }
      });
    }
    // Textures
    unsafe {
      let images = &mut (*GLOBALS.shared_images_arena);
      let im_images = w_device.imgui_renderer.textures();

      let im_w = imgui::Window::new("Images")
        .size([400.0, 400.0], Condition::FirstUseEver)
        .collapsed(false, Condition::FirstUseEver);

      im_w.build(&im_ui, || {
        let win_sz = im_ui.window_size();

        let draw_list = im_ui.get_window_draw_list();

        let mut i = 0;
        for image in images {
          im_ui.text(i.to_string());

          im_ui.invisible_button("btn".to_string() + &i.to_string(), [100.0, 100.0]);
          

          if image.1.imgui_id.id() != 0{
            draw_list
              .add_image(image.1.imgui_id, im_ui.item_rect_min(), im_ui.item_rect_max())
              .build();
          }

          i += 1;
        }
      });
    }
  }
}
