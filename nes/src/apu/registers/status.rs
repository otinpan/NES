// @trace-pilot bc2816a3039d7c96501e231be51f7f5033384e23
// Status ($4015)
bitflags!{
    pub struct StatusRegister: u8{
        const PULSE1            =0b0000_0001;
        const PULSE2            =0b0000_0010;
        const TRIANGLE          =0b0000_0100;
        const NOISE             =0b0000_1000;
        const DMC               =0b0001_0000;
        const DMC_INTERRUPT     =0b0010_0000;
        const FRAME_INTERRUPT   =0b0100_0000;
    }
}

impl StatusRegister{
    pub fn new() -> Self{
        StatusRegister::empty()
    }

    const WRITE_MASK: StatusRegister = StatusRegister::from_bits_truncate(0b0001_1111);

    pub fn set_pulse1(&mut self,flag: bool){
        self.set(StatusRegister::PULSE1,flag);
    }

    pub fn pulse1(&self) ->bool{
        self.contains(StatusRegister::PULSE1)
    }

    pub fn set_pulse2(&mut self,flag: bool){
        self.set(StatusRegister::PULSE2,flag);
    }

    pub fn pulse2(&self) ->bool{
        self.contains(StatusRegister::PULSE2)
    }

    pub fn set_triangle(&mut self,flag: bool){
        self.set(StatusRegister::TRIANGLE,flag);
    }

    pub fn triangle(&self) ->bool{
        self.contains(StatusRegister::TRIANGLE)
    }

    pub fn set_noise(&mut self,flag: bool){
        self.set(StatusRegister::NOISE,flag);
    }

    pub fn noise(&self) ->bool{
        self.contains(StatusRegister::NOISE)
    }

    pub fn set_dmc(&mut self,flag: bool){
        self.set(StatusRegister::DMC,flag);
    }

    pub fn dmc(&self) ->bool{
        self.contains(StatusRegister::DMC)
    }

    pub fn set_dmc_interrupt(&mut self, flag: bool){
        self.set(StatusRegister::DMC_INTERRUPT, flag);
    }

    pub fn dmc_interrupt(&self) -> bool{
        self.contains(StatusRegister::DMC_INTERRUPT)
    }

    pub fn set_frame_interrupt(&mut self, flag: bool){
        self.set(StatusRegister::FRAME_INTERRUPT, flag);
    }

    pub fn frame_interrupt(&self) -> bool{
        self.contains(StatusRegister::FRAME_INTERRUPT)
    }

    pub fn snapshot(&self) -> u8{
        self.bits()
    }

    pub fn update(&mut self,data: u8){
        let writable = StatusRegister::from_bits_truncate(data) & Self::WRITE_MASK;
        let preserved_interrupts = *self & !Self::WRITE_MASK;
        *self = preserved_interrupts | writable;
    }

}
