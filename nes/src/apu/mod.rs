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

    pub frame_interrupt: bool,
    pub dmc_interrupt: bool,

    pub frame_step: u8,

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

            frame_interrupt: false,
            dmc_interrupt: false,

            frame_step: 0,
            cycles: 0,
        }
    }

    pub fn clock_quater_frame(&mut self){
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
        self.triangle.clock_linear_counter();
        self.noise.clock_envelope();
    }

    pub fn clock_half_frame(&mut self){
        self.pulse1.clock_length_counter();
        self.pulse1.clock_sweep();

        self.pulse2.clock_length_counter();
        self.pulse2.clock_sweep();

        self.triangle.clock_length_counter();
        self.noise.clock_length_counter();
    }
}

impl APU for NesAPU{
    fn write_to_status(&mut self,data: u8){
        self.status.update(data);
        self.pulse1.set_enabled(self.status.pulse1());
        self.pulse2.set_enabled(self.status.pulse2());
        self.triangle.set_enabled(self.status.triangle());
        self.noise.set_enabled(self.status.noise());
        todo!("dmc")
    }

    fn read_status(&mut self) -> u8{
        let mut result=0b0000_0000;
        if self.pulse1.get_length_counter()!=0{
            result=result | 0b0000_0001;
        }

        if self.pulse2.get_length_counter()!=0{
            result=result | 0b0000_0010;
        }

        if self.triangle.get_length_counter()!=0{
            result=result | 0b0000_0100;
        }

        if self.noise.get_length_counter()!=0{
            result=result | 0b0000_1000;
        }

    
        // @trace-pilot 4e190e58eafb304aeb7eb8d9ba0cef2798debe45
        // Reading this register clears the frame interrupt flag (but not the DMC interrupt flag).
        if self.frame_interrupt{
            result=result | 0b0100_0000;
        }

        if self.dmc_interrupt{
            result=result | 0b1000_0000;
        }

        self.frame_interrupt=false;
        result
        
    }


    // @trace-pilot 8e6526576cc2192e29d85bca2829a4e775f91685
    // once each PPU frame and resets the sequence before it ever reaches step 5
    fn write_to_frame_counter(&mut self,data: u8) {
        self.frame_counter.update(data);
        if self.frame_counter.irq_inhibit(){
            self.frame_interrupt=false;
        }

        self.frame_step=0;

        if self.frame_counter.five_step_mode(){
            self.clock_quater_frame();
            self.clock_half_frame();
        }
    }

    fn read_frame_counter(&self) ->u8 {
       self.frame_counter.bits() 
    }
}
