// @trace-pilot a915cd462c3eae8b32992df74c878f74590c5ce6
// APU Triangle
use registers::linear_counter::LinearCounterRegister;
use registers::timer_high::TimerHighRegister;

pub mod registers;

pub struct TriangleChannel{
    pub linear: LinearCounterRegister,
    pub timer_low: u8,
    pub timer_high: TimerHighRegister,

    timer_counter: u16,
    sequence_step: u8,
    length_counter: u8,
    linear_counter: u8,
    reload_flag: bool,

    // @trace-pilot 1dceb0e077fe7763d877954ed20ee90510d54085
    // Status ($4015)
    enabled: bool,
}

pub trait Triangle{
    fn write_to_linear(&mut self,data: u8);
    fn write_to_timer_low(&mut self,data: u8);
    fn write_to_timer_high(&mut self,data :u8);

    fn read_linear(&self) -> u8;
    fn read_timer_low(&self) -> u8;
    fn read_timer_high(&self) -> u8;
}

impl TriangleChannel{
    pub fn new()->Self{
        TriangleChannel{
            linear: LinearCounterRegister::new(),
            timer_low: 0,
            timer_high: TimerHighRegister::new(),

            timer_counter: 0,
            sequence_step: 0,
            length_counter: 0,
            linear_counter: 0,
            reload_flag: false,

            enabled: false,
        }
    }

    pub fn timer_period(&self) -> u16{
        (((self.timer_high.timer_high() as u16) & 0b111) <<8) | self.timer_low as u16
    }

    fn set_timer_period(&mut self,period :u16){
        self.timer_low=(period  & 0x00FF) as u8;
        let high=(self.timer_high.bits() & 0b1111_1000) | (((period>>8) as u8) & 0b0000_0111);
        self.timer_high.update(high);
    }

    pub fn set_enabled(&mut self,enabled: bool){
        self.enabled=enabled;
        if !enabled{
            self.length_counter=0;
        }
    }

    pub fn get_length_counter(&self) ->u8{
        self.length_counter
    }

    pub fn tick(&mut self,cycles: u8){
        for _ in 0..cycles{
            if self.timer_counter==0{
                self.timer_counter=self.timer_period();
                if self.length_counter>0 && self.linear_counter>0{
                    self.sequence_step=(self.sequence_step+1)%32;
                }
            }else{
                self.timer_counter-=1;
            }
        }
    }

    // @trace-pilot 739279a03ca751bebdf91e229d0eeaf559b9172c
    // The sequencer sends the following looping 32-step sequence
    const SEQUENCER_TABLE: [u8;32]=[
        15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
        0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15
    ];

    // @trace-pilot add72b7576333a99fa8e8d57c3205301a112ae0d
    // If the enabled flag is set, the length counter is loaded with entry L of the length table:
    const LENGTH_TABLE:[u8;32]=[
        10,254,20,2,40,4,80,6,
        160,8,60,10,14,12,26,14,
        12,16,24,18,48,20,96,22,
        192,24,72,26,16,28,32,30,
    ];

    pub fn output(&self) ->u8{
        if !self.enabled || self.length_counter==0 || self.linear_counter==0{
            return 0;
        }

        if self.timer_period() <2{
            return 0;
        }
        Self::SEQUENCER_TABLE[self.sequence_step as usize]
    }

    pub fn clock_length_counter(&mut self){
        if !self.linear.control_flag() &&  self.length_counter>0{
            self.length_counter-=1;
        }
    }

    pub fn clock_linear_counter(&mut self){
        if self.reload_flag{
            self.linear_counter=self.linear.reload();
        }else if self.linear_counter>0{
            self.linear_counter-=1;
        }

        if !self.linear.control_flag(){
            self.reload_flag=false;
        }
    }

}

impl Triangle for TriangleChannel{
    fn write_to_linear(&mut self,data: u8){
        self.linear.update(data);
    }

    fn write_to_timer_low(&mut self,data: u8){
        self.timer_low=data;
    }

    fn write_to_timer_high(&mut self,data: u8){
        self.timer_high.update(data);
        if self.enabled{
            self.length_counter=Self::LENGTH_TABLE[self.timer_high.length_counter_load() as usize];
        }
        self.reload_flag=true;
    }

    fn read_linear(&self) -> u8{
        self.linear.bits()
    }

    fn read_timer_low(&self) -> u8{
        self.timer_low
    }

    fn read_timer_high(&self) -> u8{
        self.timer_high.bits()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn triangle() -> TriangleChannel {
        TriangleChannel::new()
    }

    #[test]
    fn test_timer_period_combines_high_and_low_bits() {
        let mut tri = triangle();
        tri.write_to_timer_low(0xAB);
        tri.write_to_timer_high(0b0000_0101);

        assert_eq!(tri.timer_period(), 0x05AB);
    }

    #[test]
    fn test_write_timer_high_loads_length_counter_and_sets_reload_flag() {
        let mut tri = triangle();
        tri.length_counter = 0;
        tri.reload_flag = false;
        tri.set_enabled(true);

        tri.write_to_timer_high(0b1111_1000);

        assert_eq!(tri.length_counter, TriangleChannel::LENGTH_TABLE[0b1_1111]);
        assert!(tri.reload_flag);
    }

    #[test]
    fn test_length_counter_decrements_only_when_control_flag_is_clear() {
        let mut tri = triangle();
        tri.length_counter = 3;
        tri.write_to_linear(0b0000_0101);

        tri.clock_length_counter();
        assert_eq!(tri.length_counter, 2);

        tri.write_to_linear(0b1000_0101);
        tri.clock_length_counter();
        assert_eq!(tri.length_counter, 2);
    }

    #[test]
    fn test_linear_counter_reloads_and_clears_reload_flag_when_control_clear() {
        let mut tri = triangle();
        tri.write_to_linear(0b0000_0101);
        tri.reload_flag = true;

        tri.clock_linear_counter();
        assert_eq!(tri.linear_counter, 5);
        assert!(!tri.reload_flag);

        tri.clock_linear_counter();
        assert_eq!(tri.linear_counter, 4);
    }

    #[test]
    fn test_linear_counter_keeps_reloading_while_control_flag_is_set() {
        let mut tri = triangle();
        tri.write_to_linear(0b1000_0111);
        tri.reload_flag = true;

        tri.clock_linear_counter();
        assert_eq!(tri.linear_counter, 7);
        assert!(tri.reload_flag);

        tri.linear_counter = 1;
        tri.clock_linear_counter();
        assert_eq!(tri.linear_counter, 7);
    }

    #[test]
    fn test_output_is_muted_when_counters_are_zero_or_period_too_small() {
        let mut tri = triangle();
        tri.set_enabled(true);
        tri.sequence_step = 4;
        tri.length_counter = 1;
        tri.linear_counter = 1;
        tri.write_to_timer_low(1);
        tri.write_to_timer_high(0);

        assert_eq!(tri.output(), 0);

        tri.write_to_timer_low(2);
        tri.length_counter = 0;
        assert_eq!(tri.output(), 0);

        tri.length_counter = 1;
        tri.linear_counter = 0;
        assert_eq!(tri.output(), 0);
    }

    #[test]
    fn test_output_uses_triangle_sequence_when_active() {
        let mut tri = triangle();
        tri.set_enabled(true);
        tri.length_counter = 1;
        tri.linear_counter = 1;
        tri.write_to_timer_low(2);
        tri.write_to_timer_high(0);

        tri.sequence_step = 0;
        assert_eq!(tri.output(), 15);

        tri.sequence_step = 15;
        assert_eq!(tri.output(), 0);

        tri.sequence_step = 31;
        assert_eq!(tri.output(), 15);
    }

    #[test]
    fn test_tick_advances_sequence_only_when_both_counters_are_non_zero() {
        let mut tri = triangle();
        tri.set_enabled(true);
        tri.write_to_timer_low(2);
        tri.write_to_timer_high(0);
        tri.timer_counter = 0;
        tri.length_counter = 1;
        tri.linear_counter = 1;

        tri.tick(1);
        assert_eq!(tri.sequence_step, 1);

        tri.timer_counter = 0;
        tri.linear_counter = 0;
        tri.tick(1);
        assert_eq!(tri.sequence_step, 1);

        tri.timer_counter = 0;
        tri.linear_counter = 1;
        tri.length_counter = 0;
        tri.tick(1);
        assert_eq!(tri.sequence_step, 1);
    }
}
