use crate::cpu::Mem;
use crate::cartridge::Rom;
use crate::ppu::NesPPU;
use crate::ppu::PPU;
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

pub struct Bus{
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: NesPPU,
}

impl Bus{
    pub fn new(rom: Rom)-> Self{
        let ppu=NesPPU::new(rom.chr_rom,rom.screen_mirroring);
        Bus{
            cpu_vram:[0; 2048],
            prg_rom: rom.prg_rom,
            ppu: ppu,
        }
    }

    fn read_prg_rom(&self,mut addr:u16)->u8{
        addr -=0x8000;
        if self.prg_rom.len()==0x4000 && addr>=0x4000{
            addr=addr%0x4000;
        }
        self.prg_rom[addr as usize]
    }
}

impl Mem for Bus{
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

            0x2008 ..=PPU_REGISTERS_MIRRORS_END =>{
                let mirror_down_addr = addr & 0b0010_0000_0000_0111;
                self.mem_read(mirror_down_addr)
            }
            0x8000..=0xffff => self.read_prg_rom(addr),
            _ =>{
                println!("Ignoreing mem access at {}",addr);
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
            0x2001 => todo!("write to mask"),
            0x2003 => todo!("write to oam"),
            0x2004 => todo!("write to oam data"),
            0x2005 => todo!("write scroll"),
            0x2006 => self.ppu.write_to_ppu_addr(data),
            0x2007 => self.ppu.write_to_data(data),
            0x2008..=PPU_REGISTERS_MIRRORS_END=>{
                let mirror_down_addr=addr & 0b0010_0000_0000_0111;
                self.mem_write(mirror_down_addr,data);
            }
            0x8000..=0xffff=>{
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ =>{
                println!("Ignoreing mem access at {}",addr);
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
        let mut bus=Bus::new(test::test_rom(vec![]));
        bus.mem_write(0x01,0x55);
        assert_eq!(bus.mem_read(0x01),0x55);
    }
}
