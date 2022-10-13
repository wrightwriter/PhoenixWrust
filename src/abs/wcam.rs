

// use nalg

use nalgebra_glm::{Vec3, Mat4, look_at, radians, vec1};


pub struct WCamera {
    pub up: Vec3,
    pub pos: Vec3,
    pub look_at: Vec3,

    pub view_mat: Mat4,
    pub proj_mat: Mat4,
    pub inv_view_mat: Mat4,
    pub inv_proj_mat: Mat4,

    pub width: u32,
    pub height: u32,
    pub aspect_ratio: f32,

    pub fov: f32,
    pub near: f32,
    pub far: f32,

}


impl WCamera{
    fn update_aspect_ratio(){
    }
    pub fn new(width: u32, height: u32)->Self{
        let pos= Vec3::new(0.0,1.0,-1.0);
        let look_at= Vec3::new(0.0,0.0,0.0);
        let view_mat= Mat4::identity();
        let proj_mat= Mat4::identity();
        let inv_view_mat= Mat4::identity();
        let inv_proj_mat= Mat4::identity();
        let up = Vec3::new(0.0,1.0,0.0);
        
        let aspect_ratio = width as f32 / height as f32;

        Self{
            pos,
            look_at,
            view_mat,
            proj_mat,
            inv_view_mat,
            inv_proj_mat,
            up,
            width,
            height,
            aspect_ratio,
            fov: 90.0,
            near: 0.01,
            far: 100.0, 
        }
    }
    
    pub fn refresh(&mut self){
        self.view_mat = look_at(&self.pos, &self.look_at, &self.up);
        self.proj_mat = nalgebra_glm::perspective(
            self.aspect_ratio, radians(&vec1(self.fov))[0], 
            self.near, self.far);
        self.proj_mat[(1, 1)] *= -1.0;

    }
}



