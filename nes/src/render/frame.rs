pub struct Frame{
    pub data: Vec<u8>,
}

impl Frame{
    // @trace-pilot 9f26b56864f0246328b6218286e31bbd9b8ec4c8
    // The PPU outputs a picture region of 256x240 pixels
    const WIDTH: usize =256;
    const HEIGHT: usize =240;

    pub fn new() -> Self{
        Frame{
            data: vec![0;(Frame::WIDTH)*(Frame::HEIGHT)*3],
        }
    }

    pub fn set_pixel(&mut self,x:usize,y:usize,rgb:(u8,u8,u8)){
        let base=y*3*Frame::WIDTH+x*3;
        if base +2 <self.data.len(){
            self.data[base]=rgb.0;
            self.data[base+1]=rgb.1;
            self.data[base+2]=rgb.2;
        }
    }
}
