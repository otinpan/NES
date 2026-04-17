// @trace-pilot 846013c8e3c27c2b6b5bc81e20cd1d5202aea6fd
// APU
pub mod pulse;
pub mod triangle;
pub mod noise;
pub mod registers;

use pulse::PulseChannel;
use triangle::TriangleChannel;
use noise::NoiseChannel;
use registers::status::StatusRegister;
use registers::frame_counter::FrameCounterRegister;

pub struct NesAPU{
    pub pulse1: PulseChannel,
    pub pulse2: PulseChannel,
    pub triangle: TriangleChannel,
    pub noise: NoiseChannel,

    pub status: StatusRegister,
    pub frame_counter: FrameCounterRegister,

    cycles: usize,
}

pub trait APU{
    fn write_to_status(&mut self,data :u8);
    fn read_status(&mut self) ->u8;
    fn write_to_frame_counter(&mut self,data: u8);
    fn read_frame_counter(&self) ->u8;
}

impl NesAPU{
    pub fn new() -> Self{
        NesAPU{
            pulse1: PulseChannel::new(true),
            pulse2: PulseChannel::new(false),
            triangle: TriangleChannel::new(),
            noise: NoiseChannel::new(),
            
            status: StatusRegister::new(),
            frame_counter: FrameCounterRegister::new(),
            cycles: 0,
        }
    }
}

impl APU for NesAPU{
    fn write_to_status(&mut self,data: u8){
        self.status.update(data);
        self.pulse1.set_enabled(self.status.pulse1());
        self.pulse2.set_enabled(self.status.pulse2());
        self.triangle.set_enabled(self.status.triangle());
        self.noise.set_enabled(self.status.noise());
        TODO!("dmc")
    }

    fn read_status(&mut self) -> u8{
        let mut result: u8;
        TODO!("update result")
    }

    fn write_to_frame_counter(&mut self,data: u8) {
    }

    fn read_frame_counter(&self) ->u8 {
    }
}
