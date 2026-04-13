use registers::control::ControlRegister;
use registers::sweep::SweepRegister;
use registers::timer_high::TimerHighRegister;

pub mod registers;

pub struct PulseChannel{
    pub ctrl: ControlRegister,
    pub sweep: SweepRegister,
    pub timer_low: u8,
    pub timer_high: TimerHighRegister,

    timer_counter: u16, // 周波数
    duty_step: u8, // 波の位置
    length_counter: u8, // 音の長さ
    envelope_decay: u8, // 音量
    envelope_divider: u8, // 音量更新タイミング
    envelope_start: bool, // 再スタートするか
    sweep_reload: bool, // sweep制御
}

pub trait Pulse{
    fn write_to_ctrl(&mut self,data: u8);
    fn write_to_sweep(&mut self,data: u8);
    fn write_to_timer_low(&mut self,data: u8);
    fn write_to_timer_high(&mut self,data: u8);
}

impl PulseChannel{
    pub fn new() -> Self{
        PulseChannel{
            ctrl: ControlRegister::new(),
            sweep: SweepRegister::new(),
            timer_low: 0b0000_0000,
            timer_high: TimerHighRegister::new(),

            timer_counter: 0,
            duty_step: 0,
            length_counter: 0,
            envelope_decay: 0,
            envelope_divider: 0,
            envelope_start: false,
            sweep_reload: false,
        }
    }

    pub fn timer_period(&self) -> u16{
        (((self.timer_high.timer_high() as u16) & 0b111)<<8) | self.timer_low as u16
    }

    pub fn tick(&mut self,cycles: u8){
        for _ in 0..cycles{
            if self.timer_counter==0{
                self.timer_counter=self.timer_period();
                self.duty_step=(self.duty_step+1)%8;
            }else{
                self.timer_counter-=1;
            }
        }
    }
    // @trace-pilot 67fb2c1c9df0bc696b53f8f34a22649b3ac6937b
    // Duty Cycle Sequences
    const DUTY_TABLE: [[u8;8];4]=[
        [0,1,0,0,0,0,0,0], // 12.5%
        [0,1,1,0,0,0,0,0], // 25%
        [0,1,1,1,1,0,0,0], // 50%
        [1,0,0,1,1,1,1,1], // 75%
    ];

    pub fn output(&self) -> u8{
        if self.length_counter==0{
            return 0;
        }
        if self.timer_period() <8{
            return 0;
        }
        let duty=self.ctrl.duty();
        let step=self.duty_step as usize;

        if Self::DUTY_TABLE[duty as usize][step]==0{
            0
        }else{
            self.ctrl.volume()
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

    pub fn clock_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_decay = 15;
            self.envelope_divider = self.ctrl.envelope_period();
        } else if self.envelope_divider == 0 {
            self.envelope_divider = self.ctrl.envelope_period();

            if self.envelope_decay == 0 {
                if self.ctrl.length_halt() {
                    self.envelope_decay = 15;
                }
            } else {
                self.envelope_decay -= 1;
            }
        } else {
            self.envelope_divider -= 1;
        }
    }

}

impl Pulse for PulseChannel{
    fn write_to_ctrl(&mut self,data: u8){
        self.ctrl.update(data);
    }

    fn write_to_sweep(&mut self,data :u8){
        self.sweep.update(data);
    }

    fn write_to_timer_low(&mut self,data: u8){
        self.timer_low=data;
    }

    fn write_to_timer_high(&mut self,data: u8){
        self.timer_high.update(data);
        self.duty_step=0;
        self.length_counter=Self::LENGTH_TABLE[self.timer_high.length_counter_load() as usize];
        self.envelope_start=true;
    }
}
