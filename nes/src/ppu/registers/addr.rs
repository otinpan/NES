// @trace-pilot e99bb37f3e374b748f68a2dace509f23ad3a6889
//v: During rendering, used for the scroll position. Outside of rendering, used as the current VRAM address.

pub struct AddrRegister{
    value: (u8,u8),
    hi_ptr: bool,
}

impl AddrRegister{
    pub fn new()->Self{
        AddrRegister{
            value: (0,0),
            hi_ptr: true,
        }
    }

    fn set(&mut self,data:u16){
        self.value.0=(data>>8) as u8;
        self.value.1=(data & 0xff) as u8;
    }
    // @trace-pilot 66cc29fef159af1d531b912d25f1636db1b6751d
    //The 16-bit address is written to PPUADDR one byte at a time, high byte first. Whether this is the first or second write is tracked by the PPU's internal w register, which is shared with PPUSCROLL.
    pub fn update(&mut self,data:u8){
        if self.hi_ptr{
            self.value.0=data;
        }else{
            self.value.1=data;
        }

        if self.get() > 0x3fff{
            self.set(self.get() & 0x3fff);
        }

        self.hi_ptr=!self.hi_ptr;
    }

    pub fn increment(&mut self,inc: u8){
        let lo=self.value.1;
        self.value.1=self.value.1.wrapping_add(inc);
        if lo > self.value.1{
            self.value.0=self.value.0.wrapping_add(1);
        }
        if self.get() > 0x3fff{
            self.set(self.get() & 0x3fff);
        }
    }

    pub fn reset_latch(&mut self){
        self.hi_ptr=true;
    }

    pub fn get(&self)->u16{
        ((self.value.0 as u16)<<8) | (self.value.1 as u16)
    }
}


