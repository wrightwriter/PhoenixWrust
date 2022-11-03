use std::{time::{Duration, SystemTime}};

use nalgebra_glm::{lerp, vec1};

#[derive(Clone, Copy)]
pub struct WTime {
  pub frame: i64,
  
  pub dt: Duration,
  pub dt_f32: f32,
  pub dt_f64: f64,
  pub dt_ns: u64,

  pub t_f32: f32,

  pub fps: f32,
  fps_internal: f32,
  s_since_fps_update: f32,
  

  pub time_since_start: Duration,
  
  start_time_engine: SystemTime,
  
  frame_start: SystemTime,
  frame_end: SystemTime,
}


impl WTime{
    pub fn new() -> Self{
        Self { 
            frame: 0, 
            dt: Duration::from_secs(0),
            dt_f32: 0.0,
            dt_f64: 0.0,
            dt_ns: 0,
            time_since_start: Duration::from_secs(0),
            start_time_engine: SystemTime::now(),
            frame_start: SystemTime::now(),
            frame_end: SystemTime::now(),
            t_f32: 0.0,
            fps: 0.0,
            s_since_fps_update: 0.0,
            fps_internal: 0.0,
        }
    }
    
    pub fn reset(&mut self){
        self.frame= 0;
        self.dt= Duration::from_secs(0);
        self.dt_f32= 0.0;
        self.dt_f64= 0.0;
        self.dt_ns= 0;
        self.t_f32= 0.0;
        self.time_since_start= Duration::from_secs(0);
        self.start_time_engine= SystemTime::now();
        self.frame_start= SystemTime::now();
        self.frame_end= SystemTime::now();
        self.fps = 0.0;
        self.fps_internal = 0.0;
        self.s_since_fps_update = 0.0;
    }
    
    pub fn tick(&mut self){
        let time_now = SystemTime::now();
        self.frame_end = time_now;
        
        self.frame += 1;
        
        self.dt = self.frame_end.duration_since(self.frame_start).unwrap(); 

        self.dt_f32 = self.dt.as_secs_f32();
        self.dt_f64 = self.dt.as_secs_f64();
        self.dt_ns = self.dt.as_nanos() as u64;
        
        self.time_since_start = time_now.duration_since(self.start_time_engine).unwrap();
        self.t_f32 = self.time_since_start.as_secs_f32();



        self.fps_internal = lerp(&vec1(self.fps_internal),&vec1(1./self.dt_f32), 0.1f32)[0];

        self.s_since_fps_update += self.dt_f32;

        if self.s_since_fps_update > 0.1 {
            self.fps = self.fps_internal;
            self.s_since_fps_update = 0.0;
        }
        
        
        self.frame_start = time_now;
    }
}