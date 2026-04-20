// @trace-pilot b42450c15a01552a1a76d881ebe5749d3a5be0e8
// APU DMC
use registers::control::ControlRegister;

pub mod registers;

pub struct DMCChannel{
    pub ctrl: ControlRegister,
    // @trace-pilot 65d636464e488a4c87b25440dc87c4c4c5bade45
    // $4011
    pub dac: u8,
    pub sample_start_addr: u8, // sampleのスタート位置
    pub sample_length: u8, // sampleの長さ(byte)
    
    pub current_addr: u16, // sampleの位置
    pub sample_divider: u16, // sampleの周期の早さ
    pub sample_counter: u16, // sampleがあと何byte残っているか
    
    pub current_bit: u8,
    pub current_buffer: u8, // 現在再生中のbuffer
    pub next_buffer: Option<u8>, // 次に再生されるbuffer

    pub silence_flag: bool,
    pub irq_flag: bool,

    pub enabled: bool,
}

pub trait DMC{
    fn write_to_ctrl(&mut self,data: u8);
    fn write_to_dac(&mut self,data :u8);
    fn write_to_sample_addr(&mut self,data :u8);
    fn write_to_sample_length(&mut self,data :u8);

    fn read_ctrl(&self) -> u8;
    fn read_dac(&self) -> u8;
    fn read_sample_addr(&self) -> u8;
    fn read_sample_length(&self) -> u8;
}

impl DMCChannel{
    pub fn new() -> Self{
        DMCChannel{
            ctrl: ControlRegister::new(),
            dac: 0,
            sample_start_addr: 0,
            sample_length: 0,

            current_addr: 0,
            sample_divider: 0,
            sample_counter: 0,

            current_bit: 0,
            current_buffer: 0,
            next_buffer: None,

            silence_flag: true,
            irq_flag: false,

            enabled: false,
        }
    }


    pub fn restart_sample(&mut self){
        self.current_addr=0xC000 | ((self.sample_start_addr as u16) << 6);
        self.sample_counter=((self.sample_length as u16) << 4) | 1;
    }

    pub fn need_sample_buffer(&self) -> bool{
        self.next_buffer.is_none() && self.sample_counter>0
    }

    pub fn is_sample_counter(&self) -> bool{
        self.sample_counter >0
    }

    pub fn push_sample_byte(&mut self,data: u8){
        self.next_buffer=Some(data);

        if self.current_addr==0xFFFF{
            self.current_addr=0x8000;
        }else{
            self.current_addr+=1;
        }

        self.sample_counter-=1;
        
        self.irq_flag=false;
        if self.sample_counter==0{
            if self.ctrl.is_loop(){
                self.restart_sample();
            }else if self.ctrl.is_irq(){
                self.irq_flag=true;
            }
        }
    }

    pub fn set_enabled(&mut self,enabled: bool){
        self.enabled=enabled;
        if enabled{
            if self.sample_counter==0{
                self.restart_sample();
            }
        }else{
            self.sample_counter=0;
            self.irq_flag=false;
        }
    }

    const DMC_RATE_TABLE: [u16;16]=[
        // @trace-pilot 50da5061607539044b3a9dea89c324675c9dc0d2
        428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84,  72,  54
    ];


    pub fn clock_sample(&mut self){
        if self.sample_divider==0{
            self.sample_divider=Self::DMC_RATE_TABLE[self.ctrl.rate_index() as usize];
            self.clock_output();
        }else{
            self.sample_divider-=1;
        }
    }

    pub fn clock_output(&mut self){
        if self.current_bit==0{
            if let Some(buffer)=self.next_buffer.take(){
                self.current_buffer=buffer;
                self.silence_flag=false;
            }else{
                self.silence_flag=true;
            }
        }

        if !self.silence_flag{
            if (self.current_buffer & 1)==1{
                if self.dac<=125{
                    self.dac+=2;
                }
            }else{
                if self.dac>=2{
                    self.dac-=2;
                }
            }
        }

        self.current_buffer >>=1;
        self.current_bit+=1;
        
        if self.current_bit>=8{
            self.current_bit=0;
        }
    }

    pub fn output(&self) ->u8{
        self.dac
    }
}

impl DMC for DMCChannel{
    fn write_to_ctrl(&mut self,data :u8){
        self.ctrl.update(data);

        if !self.ctrl.is_irq(){
            self.irq_flag=false;
        }
    }

    fn write_to_dac(&mut self,data :u8){
        self.dac=data&0x7F;
    }

    fn write_to_sample_addr(&mut self,data: u8){
        self.sample_start_addr=data;
    }

    fn write_to_sample_length(&mut self,data: u8){
        self.sample_length=data;
    }

    fn read_ctrl(&self) -> u8{
        self.ctrl.bits()
    }

    fn read_dac(&self) -> u8 {
        self.dac
    }

    fn read_sample_addr(&self) -> u8 {
        self.sample_start_addr
    }

    fn read_sample_length(&self) -> u8 {
        self.sample_length
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn dmc() -> DMCChannel {
        DMCChannel::new()
    }

    #[test]
    fn test_set_enabled_false_clears_sample_counter_and_irq_flag() {
        let mut dmc = dmc();
        dmc.sample_counter = 10;
        dmc.irq_flag = true;

        dmc.set_enabled(false);

        assert_eq!(dmc.sample_counter, 0);
        assert!(!dmc.irq_flag);
    }

    #[test]
    fn test_output_returns_dac_when_channel_is_disabled() {
        let mut dmc = dmc();
        dmc.dac = 0x3f;
        dmc.enabled = false;
        dmc.sample_counter = 0;
        dmc.silence_flag = true;

        assert_eq!(dmc.output(), 0x3f);
    }

    #[test]
    fn test_output_returns_dac_when_sample_has_ended() {
        let mut dmc = dmc();
        dmc.dac = 0x55;
        dmc.enabled = true;
        dmc.sample_counter = 0;
        dmc.silence_flag = true;

        assert_eq!(dmc.output(), 0x55);
    }

    #[test]
    fn test_output_returns_dac_while_silenced() {
        let mut dmc = dmc();
        dmc.dac = 0x12;
        dmc.enabled = true;
        dmc.sample_counter = 4;
        dmc.silence_flag = true;

        assert_eq!(dmc.output(), 0x12);
    }
}
