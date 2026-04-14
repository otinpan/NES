// @trace-pilot ed7f1010358b15d0ea2ef002838324d8da4d8e84
// $4000
bitflags!{
    // 7  bit  0
    // ---- ----
    // DDLC VVVV
    // |||| ||||
    // |||| |+++- Volume / envelope divider period
    // |||| |     (0-15)
    // |||| +---- Constant volume flag
    // ||||       (0: use envelope, 1: use constant volume set by VVVV)
    // |||+------ Length counter halt / envelope loop flag
    // |||        (0: length counter counts down normally
    // |||         1: length counter halted, envelope loops)
    // ++-------- Duty cycle
    //            (0: 12.5%, 1: 25%, 2: 50%, 3: 75%)
    pub struct ControlRegister: u8{
        const DUTY_HIGH         =0b1000_0000; // shape of wave
        const DUTY_LOW          =0b0100_0000;
        const LENGTH_HALT       =0b0010_0000; // infinite play or one-shot
        const CONSTANT_VOLUME   =0b0001_0000; // if c volume will be a constant. if clear starting
                                              // at volume 15 and lowering to 0 over time
    }
}

impl ControlRegister{
    pub fn new() -> Self{
        ControlRegister::from_bits_truncate(0b0000_0000)
    }

    pub fn volume(&self) -> u8{
        self.bits()&0b0000_1111
    }

    pub fn duty(&self) -> u8{
        (self.bits()>>6) & 0b11
    }

    pub fn length_halt(&self) -> bool{
        self.contains(ControlRegister::LENGTH_HALT)
    }

    pub fn constant_volume(&self) ->bool{
        self.contains(ControlRegister::CONSTANT_VOLUME)
    }

    pub fn envelope_period(&self) -> u8{
        self.bits() & 0b0000_1111
    }

    pub fn update(&mut self,data:u8){
        *self=ControlRegister::from_bits_truncate(data);
    }
}
