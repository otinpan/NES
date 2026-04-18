// @trace-pilot 2d1486a067ed79443d34a4084870dd8ed59a9134
// $4010

bitflags!{
    pub struct ControlRegister: u8{
        const IRQ   =0b1000_0000;
        const LOOP  =0b0100_0000;
    }
}

impl ControlRegister{
    pub fn new() -> Self{
        ControlRegister::empty()
    }

    pub fn is_irq(&self) -> bool{
        self.contains(ControlRegister::IRQ)
    }

    pub fn is_loop(&self) -> bool{
        self.contains(ControlRegister::LOOP)
    }

    pub fn rate_index(&self) -> u8{
        self.bits() & 0b0000_1111
    }

    pub fn update(&mut self,data :u8){
        *self=ControlRegister::from_bits_truncate(data & 0b1100_1111);
    }
}
