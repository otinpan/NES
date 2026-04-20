use crate::apu::pulse::Pulse;
use crate::apu::triangle::Triangle;
use crate::apu::noise::Noise;
use crate::apu::dmc::DMC;
use crate::cpu::Mem;
use crate::cartridge::Rom;
use crate::ppu::NesPPU;
use crate::ppu::PPU;
use crate::joypad::Joypad;
use crate::apu::NesAPU;
use crate::apu::APU;
use crate::audio::AudioPlayer;
//  _______________ $10000  _______________
// | PRG-ROM       |       |               |
// | Upper Bank    |       |               |
// |_ _ _ _ _ _ _ _| $C000 | PRG-ROM       |
// | PRG-ROM       |       |               |
// | Lower Bank    |       |               |
// |_______________| $8000 |_______________|
// | SRAM          |       | SRAM          |
// |_______________| $6000 |_______________|
// | Expansion ROM |       | Expansion ROM |
// |_______________| $4020 |_______________|
// | I/O Registers |       |               |
// |_ _ _ _ _ _ _ _| $4000 |               |
// | Mirrors       |       | I/O Registers |
// | $2000-$2007   |       |               |
// |_ _ _ _ _ _ _ _| $2008 |               |
// | I/O Registers |       |               |
// |_______________| $2000 |_______________|
// | Mirrors       |       |               |
// | $0000-$07FF   |       |               |
// |_ _ _ _ _ _ _ _| $0800 |               |
// | RAM           |       | RAM           |
// |_ _ _ _ _ _ _ _| $0200 |               |
// | Stack         |       |               |
// |_ _ _ _ _ _ _ _| $0100 |               |
// | Zero Page     |       |               |
// |_______________| $0000 |_______________|
//
const RAM:u16=0x0000;
const RAM_MIRRORS_END:u16=0x1FFF;
const PPU_REGISTERS:u16=0x2000;
const PPU_REGISTERS_MIRRORS_END:u16=0x3FFF;

pub struct Bus<'call>{
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: NesPPU,
    apu: NesAPU,
    audio: Option<AudioPlayer>,

    cycles: usize,
    gameloop_callback: Box<dyn FnMut(&NesPPU,&NesAPU,&mut Joypad) + 'call>,
    joypad1: Joypad,
}

impl<'a> Bus<'a>{
    pub fn new<'call,F>(rom: Rom, gameloop_callback: F) -> Bus<'call>
    where 
        F: FnMut(&NesPPU,&NesAPU,&mut Joypad) + 'call,
    {
        let ppu=NesPPU::new(rom.chr_rom,rom.screen_mirroring);
        let apu=NesAPU::new();
        Bus{
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu: ppu,
            apu: apu,
            audio: None,
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
            joypad1: Joypad::new(),
        }
    }

    pub fn new_with_audio<'call, F>(rom: Rom, gameloop_callback: F, audio: AudioPlayer) -> Bus<'call>
    where
        F: FnMut(&NesPPU, &NesAPU, &mut Joypad) + 'call,
    {
        let mut bus = Self::new(rom, gameloop_callback);
        bus.audio = Some(audio);
        bus
    }

    fn read_prg_rom(&self,mut addr:u16) -> u8{
        addr -=0x8000;
        if self.prg_rom.len()==0x4000 && addr>=0x4000{
            addr=addr%0x4000;
        }
        self.prg_rom[addr as usize]
    }

    pub fn tick(&mut self,cycles: u8){
        for _ in 0..cycles{
            self.cycles+=1;
            let new_frame=self.ppu.tick(3);
            self.apu.tick(1);

            if self.apu.dmc.need_sample_buffer(){
                let addr=self.apu.dmc.current_addr;
                let data=self.mem_read(addr);
                self.apu.dmc.push_sample_byte(data);
            }

            if let Some(audio)=self.audio.as_mut(){
                audio.tick(&self.apu);
            }

            if new_frame{
                if let Some(audio)=self.audio.as_mut(){
                    audio.flush();
                }
                (self.gameloop_callback)(&self.ppu,&mut self.apu,&mut self.joypad1);
            }
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8>{
        self.ppu.poll_nmi_interrupt()
    }
}

impl Mem for Bus<'_>{
    fn mem_read(&mut self,addr:u16)->u8{
        match addr{
            RAM ..=RAM_MIRRORS_END =>{
                let mirror_down_addr = addr & 0b0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 =>{
                panic!("Attempt to read from write-only PPU address {:x}",addr);
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            // apu
            // pulse
            0x4000 => self.apu.pulse1.read_ctrl(),
            0x4001 => self.apu.pulse1.read_sweep(),
            0x4002 => self.apu.pulse1.read_timer_low(),
            0x4003 => self.apu.pulse1.read_timer_high(),
            0x4004 => self.apu.pulse2.read_ctrl(),
            0x4005 => self.apu.pulse2.read_sweep(),
            0x4006 => self.apu.pulse2.read_timer_low(),
            0x4007 => self.apu.pulse2.read_timer_high(),
            // triangle
            0x4008 => self.apu.triangle.read_linear(),
            0x4009 =>{
                // unused
                0
            }
            0x400A => self.apu.triangle.read_timer_low(),
            0x400B => self.apu.triangle.read_timer_high(),
            // noise
            0x400C => self.apu.noise.read_ctrl(),
            0x400D =>{
                // unused
                0
            }
            0x400E => self.apu.noise.read_period(),
            0x400F => self.apu.noise.read_length(),
            // dmc
            0x4010 => self.apu.dmc.read_ctrl(),
            0x4011 => self.apu.dmc.read_dac(),
            0x4012 => self.apu.dmc.read_sample_addr(),
            0x4013 => self.apu.dmc.read_sample_length(),
            // status
            0x4015 => self.apu.read_status(),
            // frame_counter
            0x4017 => self.apu.read_frame_counter(),
            // joypad1
            0x4016 =>{
                self.joypad1.read()
            }
    
            0x2008 ..=PPU_REGISTERS_MIRRORS_END =>{
                let mirror_down_addr = addr & 0b0010_0000_0000_0111;
                self.mem_read(mirror_down_addr)
            }
            0x8000..=0xffff => self.read_prg_rom(addr),
            _ =>{
                println!("Ignoreing mem access at {}",addr);
                //panic!("Ignoreing mem access at {}",addr);
                0
            }
        }
    }

    fn mem_write(&mut self,addr:u16,data:u8){
        match addr{
            RAM ..=RAM_MIRRORS_END =>{
                let mirror_down_addr=addr & 0b0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            0x2000 => self.ppu.write_to_ctrl(data),
            0x2001 => self.ppu.write_to_mask(data),
            0x2002 => panic!("attempt to write to PPU status register"),
            0x2003 => self.ppu.write_to_oam_addr(data),
            0x2004 => self.ppu.write_to_oam_data(data),
            0x2005 => self.ppu.write_to_scroll(data),
            0x2006 => self.ppu.write_to_ppu_addr(data),
            0x2007 => self.ppu.write_to_data(data),
            // apu
            // pulse
            0x4000 => self.apu.pulse1.write_to_ctrl(data),
            0x4001 => self.apu.pulse1.write_to_sweep(data),
            0x4002 => self.apu.pulse1.write_to_timer_low(data),
            0x4003 => self.apu.pulse1.write_to_timer_high(data),
            0x4004 => self.apu.pulse2.write_to_ctrl(data),
            0x4005 => self.apu.pulse2.write_to_sweep(data),
            0x4006 => self.apu.pulse2.write_to_timer_low(data),
            0x4007 => self.apu.pulse2.write_to_timer_high(data),
            // triangle
            0x4008 => self.apu.triangle.write_to_linear(data),
            0x4009 => {
                // unused
            }
            0x400A => self.apu.triangle.write_to_timer_low(data),
            0x400B => self.apu.triangle.write_to_timer_high(data),
            // noise
            0x400C => self.apu.noise.write_to_ctrl(data),
            0x400D => {
                // unused
            }
            0x400E => self.apu.noise.write_to_period(data),
            0x400F => self.apu.noise.write_to_length(data),
            // dmc
            0x4010 => self.apu.dmc.write_to_ctrl(data),
            0x4011 => self.apu.dmc.write_to_dac(data),
            0x4012 => self.apu.dmc.write_to_sample_addr(data),
            0x4013 => self.apu.dmc.write_to_sample_length(data),
            // status
            0x4015 => self.apu.write_to_status(data),
            0x4016 =>{
                self.joypad1.write(data);
            },
            0x4017 =>{
                self.apu.write_to_frame_counter(data);
            }
            0x4014 =>{
                let mut buffer:[u8;256]=[0;256];
                let hi:u16=(data as u16)<<8;
                for i in 0..256u16{
                    buffer[i as usize]=self.mem_read(hi+i);
                }

                self.ppu.write_oam_dma(&buffer);
            }
            0x2008..=PPU_REGISTERS_MIRRORS_END=>{
                let mirror_down_addr=addr & 0b0010_0000_0000_0111;
                self.mem_write(mirror_down_addr,data);
            }
            0x8000..=0xffff=>{
                panic!("Attempt to write to Cartridge ROM space");
            }
            _ =>{
                println!("Ignoreing mem access at {}",addr);
                //panic!("Ignoreing mem access at {}", addr);
            }
        }
    }
}

#[cfg(test)]
mod test{
    use super::*;
    use crate::cartridge::test;

    #[test]
    fn test_mem_read_write_to_ram(){
        let mut bus=Bus::new(test::test_rom(vec![]), |_ppu,_apu,_joyad|{});
        bus.mem_write(0x01,0x55);
        assert_eq!(bus.mem_read(0x01),0x55);
    }
}
