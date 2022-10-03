

pub trait WPongableTrait {
//   fn new(device: &ash::Device) -> Self { Self {} }
    fn pong(&mut self);
    fn is_pongable(&mut self)->bool;
}
