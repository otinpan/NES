// @trace-pilot 846013c8e3c27c2b6b5bc81e20cd1d5202aea6fd
// APU
pub mod pulse;
pub mod triangle;
pub mod noise;
pub mod registers;
pub mod dmc;

use pulse::PulseChannel;
use triangle::TriangleChannel;
use noise::NoiseChannel;
use dmc::DMCChannel;
use registers::status::StatusRegister;
use registers::frame_counter::FrameCounterRegister;

pub struct NesAPU{
    pub pulse1: PulseChannel,
    pub pulse2: PulseChannel,
    pub triangle: TriangleChannel,
    pub noise: NoiseChannel,
    pub dmc: DMCChannel,

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
            dmc: DMCChannel::new(),

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

    pub fn need_dmc_sample_buffer(&self) -> bool{
        self.dmc.need_sample_buffer()
    }

    pub fn get_irq(&self) -> bool{
        // dmcのサンプルが終了 or mode0の最後のステップ&& !irq_inhibit()
        self.dmc.irq_flag || self.frame_interrupt
    }


    pub fn output(&self) -> u8{
        let out_pulse1=self.pulse1.output();
        let out_pulse2=self.pulse2.output();
        let out_triangle=self.triangle.output();
        let out_noise=self.noise.output();
        let out_dmc=self.dmc.output();

        (out_pulse1 + out_pulse2 + out_triangle + out_noise + out_dmc)/5
    }

    pub fn tick(&mut self,cycles: u8){
        for _ in 0..cycles{
            self.cycles+=1;

            // @trace-pilot ca3cf9e7d182c21f7f287ece494580ab59cb5c26
            // The pitch of the triangle channel is one octave below the pulse channels with an equivalent timer value (i.e. use the formula above but divide the resulting frequency by two).
            self.triangle.tick(1);

            if self.cycles%2==0{
                self.pulse1.tick(1);
                self.pulse2.tick(1);
                self.noise.tick(1);
                self.dmc.clock_sample();
            }

            self.clock_frame_counter();
        }
    }

    // @trace-pilot 7ce8e37cb4da138fc5da6230c506e53f5992fd5e
    pub fn clock_frame_counter(&mut self) {
        if self.frame_counter.five_step_mode() {
            match self.cycles {
                3729 => {
                    self.clock_quater_frame();
                }
                7457 => {
                    self.clock_quater_frame();
                    self.clock_half_frame();
                }
                11186 => {
                    self.clock_quater_frame();
                }
                14915 => {
                    // 5-step mode のこの位置では何もしない
                }
                18641 => {
                    self.clock_quater_frame();
                    self.clock_half_frame();

                    // 5-step mode は frame IRQ を出さない
                    self.cycles = 0;
                }
                _ => {}
            }
        } else {
            match self.cycles {
                3729 => {
                    self.clock_quater_frame();
                }
                7457 => {
                    self.clock_quater_frame();
                    self.clock_half_frame();
                }
                11186 => {
                    self.clock_quater_frame();
                }
                14915 => {
                    self.clock_quater_frame();
                    self.clock_half_frame();

                    if !self.frame_counter.irq_inhibit() {
                        self.frame_interrupt = true;
                    }

                    self.cycles = 0;
                }
                _ => {}
            }
        }
    }

}

impl APU for NesAPU{
    fn write_to_status(&mut self,data: u8){
        self.status.update(data);
        self.dmc.irq_flag=false;
        self.dmc_interrupt=false;
        self.pulse1.set_enabled(self.status.pulse1());
        self.pulse2.set_enabled(self.status.pulse2());
        self.triangle.set_enabled(self.status.triangle());
        self.noise.set_enabled(self.status.noise());
        self.dmc.set_enabled(self.status.dmc());
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

        if self.dmc.is_sample_counter(){
            result=result | 0b0001_0000;
        }
    
        // @trace-pilot 4e190e58eafb304aeb7eb8d9ba0cef2798debe45
        // Reading this register clears the frame interrupt flag (but not the DMC interrupt flag).
        if self.frame_interrupt{
            result=result | 0b0100_0000;
        }

        if self.dmc.irq_flag{
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

        self.cycles=0;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_status_reports_dmc_irq_from_dmc_channel() {
        let mut apu = NesAPU::new();
        apu.dmc.irq_flag = true;
        apu.dmc_interrupt = false;

        assert_eq!(apu.read_status() & 0b1000_0000, 0b1000_0000);
    }

    #[test]
    fn test_write_status_clears_dmc_irq_flag() {
        let mut apu = NesAPU::new();
        apu.dmc.irq_flag = true;
        apu.dmc_interrupt = true;

        apu.write_to_status(0b0001_0000);

        assert!(!apu.dmc.irq_flag);
        assert!(!apu.dmc_interrupt);
    }

    #[test]
    fn test_write_to_frame_counter_resets_sequence_timing() {
        let mut apu = NesAPU::new();
        apu.cycles = 14914;

        apu.write_to_frame_counter(0);
        apu.tick(1);

        assert!(!apu.frame_interrupt);
        assert_eq!(apu.cycles, 1);
    }
}
