// @trace-pilot 660fde180437a43c988b08d95b8d0c705ae71055
// APU Pulse
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
    sweep_divider: u8, // 直近sweepからのカウント数

    is_pulse1: bool, // pulse1 -> true
}

pub trait Pulse{
    fn write_to_ctrl(&mut self,data: u8);
    fn write_to_sweep(&mut self,data: u8);
    fn write_to_timer_low(&mut self,data: u8);
    fn write_to_timer_high(&mut self,data: u8);

    fn read_ctrl(&self) -> u8;
    fn read_sweep(&self) -> u8;
    fn read_timer_low(&self) -> u8;
    fn read_timer_high(&self) -> u8;
}

impl PulseChannel{
    pub fn new(is_pulse1: bool) -> Self{
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
            sweep_divider: 0,

            is_pulse1: is_pulse1,
        }
    }

    pub fn timer_period(&self) -> u16{
        (((self.timer_high.timer_high() as u16) & 0b111)<<8) | self.timer_low as u16
    }

    fn set_timer_period(&mut self,period: u16){
        self.timer_low=(period & 0x00FF) as u8;
        let high=(self.timer_high.bits() & 0b1111_1000) | (((period>>8) as u8) & 0b0000_0111);
        self.timer_high.update(high);
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
        if self.sweep_target() > 0x07FF{
            return 0;
        }
        if self.length_counter==0{
            return 0;
        }
        if self.timer_period() <8{
            return 0;
        }
        let duty=self.ctrl.duty() as usize;
        let step=self.duty_step as usize;

        if Self::DUTY_TABLE[duty as usize][step]==0{
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

    fn sweep_target(&self) -> u16{
        let period=self.timer_period();
        let shift=self.sweep.shift();

        if shift==0{
            return period;
        }

        let change=period >> shift;

        if self.sweep.negate(){
            if self.is_pulse1{
                period.saturating_sub(change+1) // pulse1
            }else{
                period.saturating_sub(change) // pulse2
            }
        }else{
            period.saturating_add(change)
        }
    }

    pub fn clock_sweep(&mut self){
        if self.sweep_reload{
            self.sweep_reload=false;
            self.sweep_divider=self.sweep.period();
        }else if self.sweep_divider==0{
            self.sweep_divider=self.sweep.period();

            if self.sweep.enable() && self.sweep.shift() >0{
                let target=self.sweep_target();

                if target <= 0x7FF{
                    self.set_timer_period(target);
                }
            }
        }else{
            self.sweep_divider-=1;
        }
    }

}

impl Pulse for PulseChannel{
    fn write_to_ctrl(&mut self,data: u8){
        self.ctrl.update(data);
    }

    fn write_to_sweep(&mut self,data :u8){
        self.sweep.update(data);
        self.sweep_reload=true;
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

    fn read_ctrl(&self) -> u8{
        self.ctrl.bits()
    }

    fn read_sweep(&self) -> u8{
        self.sweep.bits()
    }

    fn read_timer_low(&self) -> u8{
        self.timer_low
    }

    fn read_timer_high(&self)->u8{
        self.timer_high.bits()
    }
}

#[cfg(test)]
pub mod test{
    use super::*;

    fn pulse1() -> PulseChannel {
        PulseChannel::new(true)
    }

    fn pulse2() -> PulseChannel {
        PulseChannel::new(false)
    }

    #[test]
    fn test_write_timer_high_loads_length_counter_and_restarts_envelope() {
        let mut pulse = pulse1();
        pulse.length_counter = 0;
        pulse.envelope_start = false;
        pulse.duty_step = 5;

        pulse.write_to_timer_high(0b1111_1000);

        assert_eq!(pulse.length_counter, PulseChannel::LENGTH_TABLE[0b1_1111]);
        assert!(pulse.envelope_start);
        assert_eq!(pulse.duty_step, 0);
    }

    #[test]
    fn test_output_follows_duty_sequence() {
        let mut pulse = pulse1();
        pulse.write_to_ctrl(0b0001_1111);
        pulse.length_counter = 1;
        pulse.write_to_timer_low(8);
        pulse.write_to_timer_high(0);

        let expected = [0, 15, 0, 0, 0, 0, 0, 0];
        for (step, &level) in expected.iter().enumerate() {
            pulse.duty_step = step as u8;
            assert_eq!(pulse.output(), level, "unexpected output at duty step {step}");
        }
    }

    #[test]
    fn test_length_counter_decrements_only_when_not_halted() {
        let mut pulse = pulse1();
        pulse.length_counter = 3;
        pulse.write_to_ctrl(0b0000_1111);

        pulse.clock_length_counter();
        assert_eq!(pulse.length_counter, 2);

        pulse.write_to_ctrl(0b0010_1111);
        pulse.clock_length_counter();
        assert_eq!(pulse.length_counter, 2);
    }

    #[test]
    fn test_envelope_decay_and_loop_follow_control_flag() {
        let mut pulse = pulse1();
        pulse.write_to_ctrl(0b0010_0000);
        pulse.write_to_timer_high(0);

        pulse.clock_envelope();
        assert_eq!(pulse.envelope_decay, 15);

        for expected in (0..15).rev() {
            pulse.clock_envelope();
            assert_eq!(pulse.envelope_decay, expected);
        }

        pulse.clock_envelope();
        assert_eq!(pulse.envelope_decay, 15);

        let mut one_shot = pulse1();
        one_shot.write_to_ctrl(0b0000_0000);
        one_shot.write_to_timer_high(0);
        one_shot.clock_envelope();
        for _ in 0..16 {
            one_shot.clock_envelope();
        }
        assert_eq!(one_shot.envelope_decay, 0);
    }

    #[test]
    fn test_sweep_negate_differs_between_pulse1_and_pulse2() {
        let mut pulse1 = pulse1();
        pulse1.write_to_timer_low(0x00);
        pulse1.write_to_timer_high(0b0000_0100);
        pulse1.write_to_sweep(0b1000_1001);

        let mut pulse2 = pulse2();
        pulse2.write_to_timer_low(0x00);
        pulse2.write_to_timer_high(0b0000_0100);
        pulse2.write_to_sweep(0b1000_1001);

        assert_eq!(pulse1.sweep_target(), 0x01FF);
        assert_eq!(pulse2.sweep_target(), 0x0200);
    }

    #[test]
    fn test_output_is_muted_when_sweep_target_overflows_11_bits() {
        let mut pulse = pulse1();
        pulse.write_to_ctrl(0b0001_1111);
        pulse.length_counter = 1;
        pulse.duty_step = 1;
        pulse.write_to_timer_low(0xFF);
        pulse.write_to_timer_high(0b0000_0111);
        pulse.write_to_sweep(0b1001_0001);

        assert_eq!(pulse.sweep_target(), 0x0BFE);
        assert_eq!(pulse.output(), 0);
    }
}
