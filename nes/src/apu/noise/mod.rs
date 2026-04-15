// @trace-pilot c4adaf1b864408e9402af94b21c40f1e406b87ac
// APU Noise

use registers::control::ControlRegister;
use registers::period::PeriodRegister;
use registers::length::LengthRegister;

pub mod registers;

pub struct NoiseRegister{
    pub ctrl: ControlRegister,
    pub period: PeriodRegister,
    pub length: LengthRegister,

    shift_register: u16, 
    timer_counter: u16,
    length_counter: u8,
    timer: u16,

    // @trace-pilot 1dceb0e077fe7763d877954ed20ee90510d54085
    // Status ($4015)
    enabled: bool,

    envelope_start: bool,
    envelope_divider: u8, 
    envelope_decay: u8, // 音量

}

pub trait Noise{
    fn write_to_ctrl(&mut self,data :u8);
    fn write_to_period(&mut self,data :u8);
    fn write_to_length(&mut self,data :u8);
    fn read_ctrl(&self) -> u8;
    fn read_period(&self) -> u8;
    fn read_length(&self) -> u8;
}
impl NoiseRegister{
    pub fn new() -> Self{
        NoiseRegister{
            ctrl: ControlRegister::new(),
            period: PeriodRegister::new(),
            length: LengthRegister::new(),

            // @trace-pilot 7cc9a0e1cab70ca6ba4395cdd07349c856f29258
            // Linear-feedback shift register
            shift_register: 1,
            timer_counter: 0,
            length_counter: 0,
            timer: 0,
            enabled: false,

            envelope_start: true,
            envelope_decay: 0,
            envelope_divider: 0,
        }
    }

    pub fn tick(&mut self,cycles: u8){
        for _ in 0..cycles{
            if self.timer_counter==0{
                self.timer_counter=self.timer;
                self.shift();
            }else{
                self.timer_counter-=1;
            }
        }
    }

    pub fn set_enabled(&mut self,enabled: bool){
        self.enabled=enabled;
        if !enabled{
            self.length_counter=0;
        }
    }

    fn shift(&mut self){
        let bit0=self.shift_register&1;
        let tap=if self.period.mode(){
            (self.shift_register>>6) &1
        }else{
            (self.shift_register>>1) &1
        };

        let feedback=bit0^tap;
        self.shift_register=self.shift_register>>1;
        self.shift_register|=feedback<<14;
    }

    pub fn output(&self) ->u8{
        if !self.enabled || self.length_counter==0 || (self.shift_register &1)==1{
            return 0;
        }
        
        if self.ctrl.constant_volume(){
            self.ctrl.volume()
        }else{
            self.envelope_decay
        }
    }

    // @trace-pilot add72b7576333a99fa8e8d57c3205301a112ae0d
    // If the enabled flag is set, the length counter is loaded with entry L of the length table:
    const LENGTH_TABLE:[u8;32]=[
        10,254,20,2,40,4,80,6,
        160,8,60,10,14,12,26,14,
        12,16,24,18,48,20,96,22,
        192,24,72,26,16,28,32,30,
    ];

    pub fn clock_length_counter(&mut self){
        if !self.ctrl.length_halt() && self.length_counter>0{
            self.length_counter-=1;
        }
    }

    // @trace-pilot e6937281ab7e49ee2b3f43978c61e7aadd04ac0b
    // NTSC
    const NOISE_PERIOD_TABLE:[u16;16]=[
        4,8,16,32,64,96,128,160,202,254,
        380,508,762,1016,2034,4068,
    ];

    pub fn clock_envelope(&mut self){
        if self.envelope_start{
            self.envelope_start=false;
            self.envelope_decay=15;
            self.envelope_divider=self.ctrl.volume();
        }else{
            if self.envelope_divider >0{
                self.envelope_divider-=1;
            }else{
                self.envelope_divider=self.ctrl.volume();
                if self.envelope_decay>0{
                    self.envelope_decay-=1;
                }else if self.ctrl.length_halt(){
                    self.envelope_decay=15;
                }
            }
        }
    }

}

impl Noise for NoiseRegister{
    fn write_to_ctrl(&mut self,data :u8){
        self.ctrl.update(data);
    }

    fn write_to_period(&mut self,data :u8){
        self.period.update(data);
        let index=self.period.period();
        self.timer=Self::NOISE_PERIOD_TABLE[index as usize] as u16;
    }

    fn write_to_length(&mut self,data: u8){
        self.length.update(data);
        let index=self.length.length_counter_load();
        if self.enabled{
            self.length_counter=Self::LENGTH_TABLE[index as usize];
        }
        self.envelope_start=true;
    }

    fn read_ctrl(&self) -> u8{
        self.ctrl.bits()
    }

    fn read_period(&self) -> u8{
        self.period.bits()
    }

    fn read_length(&self) -> u8{
        self.length.bits()
    }


}

#[cfg(test)]
mod test {
    use super::*;

    fn noise() -> NoiseRegister {
        NoiseRegister::new()
    }

    #[test]
    fn test_write_to_period_uses_noise_period_table() {
        let mut noise = noise();

        noise.write_to_period(0b1000_1010);

        assert!(noise.period.mode());
        assert_eq!(noise.timer, NoiseRegister::NOISE_PERIOD_TABLE[0b1010]);
    }

    #[test]
    fn test_write_to_length_loads_counter_only_when_enabled_and_restarts_envelope() {
        let mut noise = noise();
        noise.envelope_start = false;

        noise.write_to_length(0b1111_1000);
        assert_eq!(noise.length_counter, 0);
        assert!(noise.envelope_start);

        noise.set_enabled(true);
        noise.envelope_start = false;
        noise.write_to_length(0b1111_1000);

        assert_eq!(noise.length_counter, NoiseRegister::LENGTH_TABLE[0b1_1111]);
        assert!(noise.envelope_start);
    }

    #[test]
    fn test_length_counter_decrements_only_when_not_halted() {
        let mut noise = noise();
        noise.length_counter = 3;
        noise.write_to_ctrl(0b0000_1111);

        noise.clock_length_counter();
        assert_eq!(noise.length_counter, 2);

        noise.write_to_ctrl(0b0010_1111);
        noise.clock_length_counter();
        assert_eq!(noise.length_counter, 2);
    }

    #[test]
    fn test_output_is_muted_when_disabled_length_counter_zero_or_shift_lsb_set() {
        let mut noise = noise();
        noise.write_to_ctrl(0b0001_1010);
        noise.envelope_decay = 7;

        assert_eq!(noise.output(), 0);

        noise.set_enabled(true);
        assert_eq!(noise.output(), 0);

        noise.length_counter = 1;
        assert_eq!(noise.output(), 0);

        noise.shift_register = 0b0010;
        assert_eq!(noise.output(), 10);

        noise.write_to_ctrl(0b0000_1111);
        assert_eq!(noise.output(), 7);
    }

    #[test]
    fn test_envelope_decay_and_loop_follow_control_flag() {
        let mut channel = noise();
        channel.write_to_ctrl(0b0010_0000);
        channel.write_to_length(0);

        channel.clock_envelope();
        assert_eq!(channel.envelope_decay, 15);

        for expected in (0..15).rev() {
            channel.clock_envelope();
            assert_eq!(channel.envelope_decay, expected);
        }

        channel.clock_envelope();
        assert_eq!(channel.envelope_decay, 15);

        let mut one_shot = noise();
        one_shot.write_to_ctrl(0b0000_0000);
        one_shot.write_to_length(0);
        one_shot.clock_envelope();
        for _ in 0..16 {
            one_shot.clock_envelope();
        }

        assert_eq!(one_shot.envelope_decay, 0);
    }
}
