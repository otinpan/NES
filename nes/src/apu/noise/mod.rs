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
        if self.length_counter==0{
            return 0;
        }
        if (self.shift_register & 1)==1{
            return 0;
        }
        
        self.ctrl.volume()
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
        self.length_counter=Self::LENGTH_TABLE[index as usize];
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

