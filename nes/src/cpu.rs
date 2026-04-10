use std::collections::HashMap;
use crate::opcodes;
use crate::bus::Bus;

bitflags! {
    /// # Status Register (P) http://wiki.nesdev.com/w/index.php/Status_flags
    ///
    ///  7 6 5 4 3 2 1 0
    ///  N V _ B D I Z C
    ///  | |   | | | | +--- Carry Flag
    ///  | |   | | | +----- Zero Flag
    ///  | |   | | +------- Interrupt Disable
    ///  | |   | +--------- Decimal Mode (not used on NES)
    ///  | |   +----------- Break Command
    ///  | +--------------- Overflow Flag
    ///  +----------------- Negative Flag
    ///
    pub struct CpuFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIV           = 0b10000000;
    }
}

const STACK: u16=0x0100;
const STACK_RESET:u8=0xfd;

pub struct CPU<'a>{
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: CpuFlags,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub bus: Bus<'a>,
}


#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode{
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

pub trait Mem{
    fn mem_read(&mut self,addr:u16)->u8;

    fn mem_write(&mut self,addr:u16,data:u8);

    fn mem_read_u16(&mut self,pos:u16)->u16{
        let lo=self.mem_read(pos) as u16;
        let hi=self.mem_read(pos+1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn mem_write_u16(&mut self,pos:u16,data:u16){
        let hi=(data>>8) as u8;
        let lo=(data&0xff) as u8;
        self.mem_write(pos,lo);
        self.mem_write(pos+1,hi);
    }
}

impl Mem for CPU<'_>{
    fn mem_read(&mut self,addr:u16)->u8{
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self,addr:u16,data:u8){
        self.bus.mem_write(addr,data);
    }

    fn mem_read_u16(&mut self, pos:u16) -> u16{
        self.bus.mem_read_u16(pos)
    }

    fn mem_write_u16(&mut self,pos:u16,data:u16){
        self.bus.mem_write_u16(pos,data);
    }
}

fn page_cross(addr1: u16,addr2: u16)->bool{
    addr1 & 0xFF00 != addr2 & 0xFF00
}

mod interrupt{
    #[derive(PartialEq,Eq)]
    pub enum InterruptType{
        NMI,
    }

    #[derive(PartialEq,Eq)]
    pub(super) struct Interrupt{
        pub(super) itype: InterruptType,
        pub(super) vector_addr: u16,
        pub(super) b_flag_mask: u8,
        pub(super) cpu_cycles: u8,
    }

    pub(super) const NMI: Interrupt=Interrupt{
        itype: InterruptType::NMI,
        vector_addr: 0xfffA,
        b_flag_mask: 0b00100000,
        cpu_cycles: 2,
    };
}

impl<'a> CPU<'a>{
    pub fn new<'b>(bus: Bus<'b>)->CPU<'b>{
        CPU{
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status:CpuFlags::from_bits_truncate(0b100100),
            program_counter: 0,
            stack_pointer: STACK_RESET,
            bus: bus,
        }
    }

    pub fn get_absolute_address(&mut self, mode: &AddressingMode, addr: u16) -> u16 {
        match mode {
            AddressingMode::ZeroPage => self.mem_read(addr) as u16,

            AddressingMode::Absolute => self.mem_read_u16(addr),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(addr);
                let addr = pos.wrapping_add(self.register_x) as u16;
                addr
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(addr);
                let addr = pos.wrapping_add(self.register_y) as u16;
                addr
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(addr);
                let addr = base.wrapping_add(self.register_x as u16);
                addr
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(addr);
                let addr = base.wrapping_add(self.register_y as u16);
                addr
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(addr);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(addr);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            _ => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn get_operand_address(&mut self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            _ => self.get_absolute_address(mode, self.program_counter),
        }
    }

    fn add_to_register_a(&mut self,data: u8){
        let sum=self.register_a as u16 + data as u16
            + (if self.status.contains(CpuFlags::CARRY){
                1
            }else{
                0
            }) as u16;

        let carry=sum>0xff;

        if carry{
            self.status.insert(CpuFlags::CARRY);
        }else{
            self.status.remove(CpuFlags::CARRY);
        }

        let result=sum as u8;

        // @trace-pilot 77652b9e547bd58e9c09b9544fbbf7cc0cc43c62
        //Overflow（Vフラグ）が立つかどうかを判定している
        if (data^result) & (result ^ self.register_a) & 0x80!=0{
            self.status.insert(CpuFlags::OVERFLOW);
        }else{
            self.status.remove(CpuFlags::OVERFLOW);
        }

        self.set_register_a(result);
    }

    fn set_register_a(&mut self,data: u8){
        self.register_a=data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // @trace-pilot 90485b0ea9b78aac401a6181aa1f5421be9a128a
    //ADC - Add with Carry
    fn adc(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(mode);
        let value=self.mem_read(addr);
        self.add_to_register_a(value);
    }

    // @trace-pilot 137869797cc95dd28e20b84e8b563fe36eb6de06
    //AND - Logical AND
    fn and(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(mode);
        let value=self.mem_read(addr);
        self.set_register_a(value & self.register_a);
    }

    // @trace-pilot 7a933e251fbf53ff68055080726eb815bd2fd68b
    //ASL - Arithmetic Shift Left
    fn asl_accumulator(&mut self){
        let mut data=self.register_a;
        self.status.set(CpuFlags::CARRY,data&0b1000_0000 !=0);
        data=data<<1;
        self.set_register_a(data);
    }
    fn asl(&mut self,mode: &AddressingMode)->u8{
        let addr=self.get_operand_address(mode);
        let mut data=self.mem_read(addr);

        self.status.set(CpuFlags::CARRY,data&0b10000000 > 0);
        data=data<<1;
        self.mem_write(addr,data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn branch(&mut self,ok: bool){
        if ok{
            let jump: i8=self.mem_read(self.program_counter) as i8;
            let jump_addr=self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);

            self.program_counter=jump_addr;
        }
    }

    // @trace-pilot 343761648c44b84449137d870daf244a9bb1bffe
    //BCC - Branch if Carry Clear
    fn bcc(&mut self){
        self.branch(!self.status.contains(CpuFlags::CARRY));
    }

    // @trace-pilot e92b977fd1f73b3926c7f92242b2e26fd88fa801
    //BCS - Branch if Carry Set
    fn bcs(&mut self){
        self.branch(self.status.contains(CpuFlags::CARRY));
    }

    // @trace-pilot 007fc863e43f6a6581af1bf31ba1130de80590b0
    //BEQ - Branch if Equal
    fn beq(&mut self){
        self.branch(self.status.contains(CpuFlags::ZERO));
    }

    // @trace-pilot 9656c0bf1c1bac3402a57d9a0a216f41592e9406
    //BIT - Bit Test
    fn bit(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(mode);
        let value=self.mem_read(addr);
        let and=self.register_a & value;
        if and==0{
            self.status.insert(CpuFlags::ZERO);
        }else{
            self.status.remove(CpuFlags::ZERO);
        }

        self.status.set(CpuFlags::NEGATIV, value&0b10000000 >0);
        self.status.set(CpuFlags::OVERFLOW, value&0b01000000 >0);
    }

    // @trace-pilot ca6697602274e0dcfd3b358803763a037bd8749b
    //BMI - Branch if Minus
    fn bmi(&mut self){
        self.branch(self.status.contains(CpuFlags::NEGATIV));
    }

    // @trace-pilot eb1cc6ffd5395f2f67932af66647e4d0b50f9851
    //BNE - Branch if Not Equal
    fn bne(&mut self){
        self.branch(!self.status.contains(CpuFlags::ZERO));
    }

    // @trace-pilot 61f9def86f7357fd55cd29f79c824326b62da6b9
    //BPL - Branch if Positive
    fn bpl(&mut self){
        self.branch(!self.status.contains(CpuFlags::NEGATIV));
    }

    // @trace-pilot 29e2fb5f531839fb2f721a59c739c18f3ca0dd4a
    //BRK - Force Interrupt
    fn brk(){
        return;
    }

    // @trace-pilot 479c078d899d80c3bb54ce7bc8bc451cdbf81e50
    //BVC - Branch if Overflow Clear
    fn bvc(&mut self){
        self.branch(!self.status.contains(CpuFlags::OVERFLOW));
    }

    // @trace-pilot d3b3a6690d29879f5c7e7be57cc3fb46a3e0c3e9
    //BVS - Branch if Overflow Set
    fn bvs(&mut self){
        self.branch(self.status.contains(CpuFlags::OVERFLOW));
    }

    // @trace-pilot 0367e42cea0240c23b278b28e386d63f3354e06c
    //CLC - Clear Carry Flag
    fn clc(&mut self){
        self.status.remove(CpuFlags::CARRY);
    }

    // @trace-pilot 5d744c6abed9d44f4fdb5cf64832525ed3c60c62
    //CLD - Clear Decimal Mode
    fn cld(&mut self){
        self.status.remove(CpuFlags::DECIMAL_MODE);
    }

    // @trace-pilot 70b501d421d3e9f2d3ee205e2a8d10977abdc8e8
    //CLI - Clear Interrupt Disable
    fn cli(&mut self){
        self.status.remove(CpuFlags::INTERRUPT_DISABLE);
    }

    // @trace-pilot 4d7cb720cf5027ef308861ba616dd86e8a71b25a
    //CLV - Clear Overflow Flag
    fn clv(&mut self){
        self.status.remove(CpuFlags::OVERFLOW);
    }

    fn compare(&mut self,mode: &AddressingMode, target: u8){
        let addr=self.get_operand_address(mode);
        let value=self.mem_read(addr);
        self.status.set(CpuFlags::CARRY,target>=value);
        self.update_zero_and_negative_flags(target.wrapping_sub(value));
    }

    //// @trace-pilot b4cefbeb21d27106366dbb34829f08d754a2fd2d
    //CMP - Compare
    fn cmp(&mut self,mode: &AddressingMode){
        self.compare(mode,self.register_a);
    }

    // @trace-pilot ea4f1767c9b1091f6d2085e922947e5216be696a
    //CPX - Compare X Register
    fn cpx(&mut self,mode: &AddressingMode){
        self.compare(mode,self.register_x);
    }

    // @trace-pilot a206047cdc425c7156ac516fc4f749e3082916d0
    //CPY - Compare Y Register
    fn cpy(&mut self,mode: &AddressingMode){
        self.compare(mode,self.register_y);
    }

    // @trace-pilot 5ef814f7abff8e10797b4f8e5bd8eb015ad242e9
    //DEC - Decrement Memory
    fn dec(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(mode);
        let mut value=self.mem_read(addr);
        value=value.wrapping_sub(1);
        self.mem_write(addr,value);
        self.update_zero_and_negative_flags(value);
    }

    // @trace-pilot 4ee51d00443dd033eb4cbfd252e0412784220e29
    //DEX - Decrement X Register
    fn dex(&mut self){
        self.register_x=self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    // @trace-pilot 95345c6863f54f1941902f884391a4ca7addad59
    //DEY - Decrement Y Register
    fn dey(&mut self){
        self.register_y=self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    // @trace-pilot 4d37fc92dff0d0272ec1baf3da3bdad1842d4208
    //EOR - Exclusive OR
    fn eor(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(mode);
        let value=self.mem_read(addr);
        self.set_register_a(self.register_a ^ value);
    }

    // @trace-pilot 0cb44bf42739c3b5f2c78d8f1993701c13b335ba
    //INC - Increment Memory
    fn inc(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(mode);
        let mut value=self.mem_read(addr);
        value=value.wrapping_add(1);
        self.mem_write(addr,value);
        self.update_zero_and_negative_flags(value);
    }

    // @trace-pilot 2d3ce5a9c8d8400965ac98972fb93f142af4ddc5
    //INX - Increment X Register
    fn inx(&mut self){
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    // @trace-pilot 9b5de2f07f963fe86d84c52d391ba8e28a946858
    //INY - Increment Y Register
    fn iny(&mut self){
        self.register_y=self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn stack_pop(&mut self) -> u8{
        self.stack_pointer=self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_pop_u16(&mut self)->u16{
        let lo=self.stack_pop() as u16;
        let hi=self.stack_pop() as u16;
        hi<<8 | lo
    }

    fn stack_push(&mut self,data: u8){
        self.mem_write((STACK as u16) +  self.stack_pointer as u16,data);
        self.stack_pointer=self.stack_pointer.wrapping_sub(1);
    }

    fn stack_push_u16(&mut self,data: u16){
        let hi=(data>>8) as u8;
        let lo=(data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    // @trace-pilot 54bd09a5822ef3b00c50dbc808c9bea4423f7f6d
    //JSR - Jump to Subroutine
    fn jsr(&mut self){
        self.stack_push_u16(self.program_counter+2-1);
        let target_addr=self.mem_read_u16(self.program_counter);
        self.program_counter=target_addr;
    }


    // @trace-pilot 6d1da098dde234e175b50d6ab68bffb4340656d8
    //LDY - Load Y Register
    fn ldy(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(mode);
        let value=self.mem_read(addr);
        self.register_y=value;
        self.update_zero_and_negative_flags(self.register_y);
    }

    // @trace-pilot 13549a87a60f90d950dce2c1e5a26dbdf6587504
    //LDX - Load X Register
    fn ldx(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(mode);
        let value=self.mem_read(addr);
        self.register_x=value;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // @trace-pilot 97f67b7a482e5af3b884196f758a8bd7c506e931
    //LDA - Load Accumulator    
    fn lda(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(mode);
        let value=self.mem_read(addr);

        self.register_a=value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    // @trace-pilot 03ff5d52e027dc6a4e4b7d90f84495a42944ee41
    //LSR - Logical Shift Right
    fn lsr_accumulator(&mut self){
        let mut data=self.register_a;
        self.status.set(CpuFlags::CARRY,data&0b0000_0001!=0);
        data=data>>1;
        self.set_register_a(data);
    }
    fn lsr(&mut self,mode: &AddressingMode)->u8{
        let addr=self.get_operand_address(mode);
        let mut data=self.mem_read(addr);
        self.status.set(CpuFlags::CARRY,data&0b0000_0001!=0);
        data=data>>1;
        self.mem_write(addr,data);
        self.update_zero_and_negative_flags(data);
        data
    }

    // @trace-pilot 8da6a74bef722d9fd2ac51d0ca738a43da1810a7
    //NOP - No Operation

    // @trace-pilot bbf91a9ab7f7516a49cfbea214c2553228f67657
    //ORA - Logical Inclusive OR
    fn ora(&mut self,mode:&AddressingMode){
        let addr=self.get_operand_address(mode);
        let data=self.mem_read(addr);
        self.set_register_a(self.register_a | data);
    }

    // @trace-pilot b35bb455abf58c5ea9d4610d216948308790a087
    //PHA - Push Accumulator
    fn pha(&mut self){
        self.stack_push(self.register_a);
    }

    // @trace-pilot 4978f20fbaafb4e4862962b5531dd101987b6cfc
    //PHP - Push Processor Status
    fn php(&mut self){
        let mut flags=self.status.clone();
        // @trace-pilot 0d195afd2b2df6c968cb2c15205cd178b1fbd53f
        //ステータスレジスタをスタックに保存する命令
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BREAK2);
        self.stack_push(flags.bits());
    }

    // @trace-pilot e7d76f7fcfe839e3ea47312d15652f9de594958e
    //PLA - Pull Accumulator
    fn pla(&mut self){
        let data=self.stack_pop();
        self.set_register_a(data);
    }

    // @trace-pilot f84ce402a281cf8fad60e919c757d460cd29d3f7
    //PLP - Pull Processor Status
    fn plp(&mut self){
        self.status.bits=self.stack_pop();
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::BREAK2);
    }

    // @trace-pilot be88d08f993bed43da26de87b69ca6469280f7ce
    //ROL - Rotate Left
    fn rol_accumulator(&mut self){
        let mut data=self.register_a;
        let old_carry=self.status.contains(CpuFlags::CARRY);

        self.status.set(CpuFlags::CARRY,data&0b1000_0000!=0);
        data=data<<1;
        if old_carry{
            data = data | 0b0000_0001;
        }

        self.set_register_a(data);
    }
    fn rol(&mut self,mode:&AddressingMode)->u8{
        let addr=self.get_operand_address(mode);
        let mut data=self.mem_read(addr);
        let old_carry=self.status.contains(CpuFlags::CARRY);
        self.status.set(CpuFlags::CARRY,data&0b1000_0000!=0);
        data=data<<1;
        if old_carry{
            data = data | 0b0000_0001;
        }

        self.mem_write(addr,data);
        self.update_zero_and_negative_flags(data);
        data
    }

    // @trace-pilot 25ab915c2c989004e20cdda38052331b49affda6
    //ROR - Rotate Right
    fn ror_accumulator(&mut self){
        let mut data=self.register_a;
        let old_carry=self.status.contains(CpuFlags::CARRY);
        self.status.set(CpuFlags::CARRY,data&0b0000_0001!=0);
        data=data>>1;
        if old_carry{
            data = data | 0b1000_0000;
        }
        self.set_register_a(data);
    }
    fn ror(&mut self,mode:&AddressingMode)->u8{
        let addr=self.get_operand_address(mode);
        let mut data=self.mem_read(addr);
        let old_carry=self.status.contains(CpuFlags::CARRY);
        self.status.set(CpuFlags::CARRY,data&0b0000_0001!=0);
        data=data>>1;
        if old_carry{
            data = data | 0b1000_0000;
        }
        self.mem_write(addr,data);
        self.update_zero_and_negative_flags(data);
        data
    }

    // @trace-pilot d57d6ca950c3448cc608c25f2e45cef757f34183
    //RTI - Return from Interrupt
    fn rti(&mut self){
        self.status.bits=self.stack_pop();
        self.status.remove(CpuFlags::BREAK);
        self.status.insert(CpuFlags::BREAK2);
        self.program_counter=self.stack_pop_u16();
    }

    // @trace-pilot 90ecb3590e46c219235bb36c5b023d7b47c8ac48
    //RTS - Return from Subroutine
    fn rts(&mut self){
        self.program_counter=self.stack_pop_u16()+1;
    }

    // @trace-pilot a8e6e1c6d12c9c20792dc7ed2684d9b742be7a94
    //SBC - Subtract with Carry
    fn sbc(&mut self,mode:&AddressingMode){
        let addr=self.get_operand_address(mode);
        let data=self.mem_read(addr);
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    // @trace-pilot f40d0b6a877722e99850d743a916f9fec6db3956
    //SEC - Set Carry Flag
    fn sec(&mut self){
        self.status.insert(CpuFlags::CARRY);
    }

    // @trace-pilot 944c52b1687668b9fb6cfafde34ca3b9c50e1e1c
    //SED - Set Decimal Flag
    fn sed(&mut self){
        self.status.insert(CpuFlags::DECIMAL_MODE);
    }

    // @trace-pilot 385cb3639eef9fea12c37bbab3cbb275b74b84c0
    //SEI - Set Interrupt Disable
    fn sei(&mut self){
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);
    }


    // @trace-pilot 20d65492dc2d5d147747320da24e9ffa6b1121db
    //STY - Store Y Register
    fn sty(&mut self, mode:&AddressingMode){
        let addr=self.get_operand_address(mode);
        self.mem_write(addr,self.register_y);
    }

    // @trace-pilot 4726862d045e45bd5e96a67fced939a99436f72a
    //STX - Store X Register
    fn stx(&mut self,mode: &AddressingMode){
        let addr=self.get_operand_address(&mode);
        self.mem_write(addr,self.register_x);
    }

    // @trace-pilot 2156e684e72895dfc4771d9f57e8138568e5bd5c
    //STA - Store Accumulator
    fn sta(&mut self,mode:&AddressingMode){
        let addr=self.get_operand_address(mode);
        self.mem_write(addr,self.register_a);
    }

    // @trace-pilot 6ffa0bf576c2d0eb2fb363c931833249f6aaa790
    //TAX - Transfer Accumulator to X
    fn tax(&mut self){
        self.register_x=self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // @trace-pilot a0830064d4415d72e3b3cbc3d65697d8de2dc38d
    //TAY - Transfer Accumulator to Y
    fn tay(&mut self){
        self.register_y=self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    // @trace-pilot 3594dcb9f68c916bc1d549e305dad0ceb8c65a49
    //TSX - Transfer Stack Pointer to X
    fn tsx(&mut self){
        self.register_x=self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    // @trace-pilot 26c87dc1cc5380c0249053854d471e1d070281ec
    //TXA - Transfer X to Accumulator
    fn txa(&mut self){
        self.set_register_a(self.register_x);
    }

    // @trace-pilot 75a0f34f8d91e120ab145dd03f0924e24cfd9748
    //TXS - Transfer X to Stack Pointer
    fn txs(&mut self){
        self.stack_pointer=self.register_x;
    }

    // @trace-pilot 10a2dde498c7eb9dfaab334b72f45f59e159b9ff
    //TYA - Transfer Y to Accumulator
    fn tya(&mut self){
        self.set_register_a(self.register_y);
    }

    fn update_zero_and_negative_flags(&mut self,result: u8){
        self.status.set(CpuFlags::ZERO,result==0);
        self.status.set(CpuFlags::NEGATIV,result&0b1000_0000 !=0);
    }


    pub fn load_and_run(&mut self,program: Vec<u8>){
        self.load(program);
        self.program_counter=0x0000;
        self.run();
    }

    pub fn load(&mut self,program: Vec<u8>){
        for i in 0 ..(program.len() as u16){
            self.mem_write(0x0000 + i,program[i as usize]);
        }
    }

    pub fn reset(&mut self){
        self.register_a=0;
        self.register_x=0;
        self.register_y=0;
        self.stack_pointer=STACK_RESET;
        self.status=CpuFlags::from_bits_truncate(0b100100);

        self.program_counter=self.mem_read_u16(0xFFFC);
    }
    pub fn run(&mut self){
        self.run_with_callback(|_|{});
    }

    fn interrupt(&mut self,interrupt: interrupt::Interrupt){
        self.stack_push_u16(self.program_counter);
        let mut flag=self.status.clone();
        flag.set(CpuFlags::BREAK,interrupt.b_flag_mask & 0b010000==1);
        flag.set(CpuFlags::BREAK2,interrupt.b_flag_mask & 0b100000==1);

        self.stack_push(flag.bits);
        self.status.insert(CpuFlags::INTERRUPT_DISABLE);
        
        self.bus.tick(interrupt.cpu_cycles);
        self.program_counter=self.mem_read_u16(interrupt.vector_addr);
    }

    pub fn run_with_callback<F>(&mut self,mut callback: F)
        where 
            F:FnMut(&mut CPU),
    {
        let ref opcodes: HashMap<u8, &'static opcodes::OpCode> = *opcodes::OPCODES_MAP;

        loop {
            if let Some(_nmi) = self.bus.poll_nmi_status(){
                self.interrupt(interrupt::NMI);
            }
            callback(self);
            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let opcode = opcodes.get(&code).expect(&format!("OpCode {:x} is not recognized", code));

            match code {
                // ADC
                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 =>{
                    self.adc(&opcode.mode);
                }

                // AND
                0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 =>{
                    self.and(&opcode.mode);
                }

                // ASL
                0x0a => self.asl_accumulator(),
                
                0x06 | 0x16 | 0x0e | 0x1e =>{
                    self.asl(&opcode.mode);
                }

                // BCC
                0x90 => self.bcc(),
                

                // BCS
                0xb0 => self.bcs(),
            

                // BEQ
                0xf0 => self.beq(),


                // BIT
                0x24 | 0x2c =>{
                    self.bit(&opcode.mode);
                }

                // BMI
                0x30 => self.bmi(),
                

                // BNE
                0xd0 => self.bne(),

                // BPL
                0x10 => self.bpl(),
                

                // BVC
                0x50 => self.bvc(),

                // BVS
                0x70 => self.bvs(),

                // CLC
                0x18 => self.clc(),

                // CLD
                0xd8 => self.cld(),

                // CLI
                0x58 => self.cli(),

                // CLV
                0xb8 => self.clv(),

                // CMP
                0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 =>{
                    self.cmp(&opcode.mode);
                }

                // CPX
                0xe0 | 0xe4 | 0xec =>{
                    self.cpx(&opcode.mode);
                }

                // CPY
                0xc0 | 0xc4 | 0xcc =>{
                    self.cpy(&opcode.mode);
                }

                // DEC
                0xc6 | 0xd6 | 0xce | 0xde =>{
                    self.dec(&opcode.mode);
                }

                // DEX
                0xca => self.dex(),

                // DEY
                0x88 => self.dey(),

                // EOR
                0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 =>{
                    self.eor(&opcode.mode);
                }

                // INC
                0xe6 | 0xf6 | 0xee | 0xfe =>{
                    self.inc(&opcode.mode);
                }

                // INX
                0xe8 => self.inx(),

                // INY
                0xc8 => self.iny(),

                // JMP
                // Absolute
                0x4c =>{
                    let mem_address=self.mem_read_u16(self.program_counter);
                    self.program_counter=mem_address;
                }
                // Indirect
                0x6c =>{
                    let mem_address=self.mem_read_u16(self.program_counter);
                    let indirect_ref= if mem_address & 0x00FF == 0x00FF{
                        let lo=self.mem_read(mem_address);
                        let hi=self.mem_read(mem_address & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    }else{
                        self.mem_read_u16(mem_address)
                    };

                    self.program_counter=indirect_ref;
                }

                // JSR
                0x20 => self.jsr(),

                // LDA
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 =>{
                    self.lda(&opcode.mode);
                }

                // LDX
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe =>{
                    self.ldx(&opcode.mode);
                }

                // LDY
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc =>{
                    self.ldy(&opcode.mode);
                }

                // LSR
                0x4a => self.lsr_accumulator(),

                0x46 | 0x56 | 0x4e | 0x5e =>{
                    self.lsr(&opcode.mode);
                }

                // NOP
                0xea => {

                }

                // ORA
                0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 =>{
                    self.ora(&opcode.mode);
                }

                // PHA
                0x48 => self.pha(),

                // PHP
                0x08 => self.php(),

                // PLA
                0x68 => self.pla(),

                // PLP
                0x28 => self.plp(),

                // ROL
                0x2a => self.rol_accumulator(),

                0x26 | 0x36 | 0x2e | 0x3e =>{
                    self.rol(&opcode.mode);
                }

                // ROR
                0x6a => self.ror_accumulator(),

                0x66 | 0x76 | 0x6e | 0x7e =>{
                    self.ror(&opcode.mode);
                }

                // RTI
                0x40 => self.rti(),

                // RTS
                0x60 => self.rts(),

                // SBC
                0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 =>{
                    self.sbc(&opcode.mode);
                }

                // SEC
                0x38 => self.sec(),

                // SED
                0xf8 => self.sed(),

                // SEI
                0x78 => self.sei(),

                // STA 
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => {
                    self.sta(&opcode.mode);
                }

                // STX
                0x86 | 0x96 | 0x8e =>{
                    self.stx(&opcode.mode);
                }

                // STY
                0x84 | 0x94 | 0x8c =>{
                    self.sty(&opcode.mode);
                }

                // TAX
                0xaa => self.tax(),

                // TAY
                0xa8 => self.tay(),

                // TSX
                0xba => self.tsx(),

                // TXA
                0x8a => self.txa(),

                // TXS
                0x9a => self.txs(),

                // TYA
                0x98 => self.tya(),
                
                0x00 => return,
                _ => todo!(),
            }

            self.bus.tick(opcode.cycles);

            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cartridge::test;
    use crate::ppu::NesPPU;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let bus=Bus::new(test::test_rom(vec![]), |ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);

        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 5);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
        assert!(cpu.status.bits() & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let bus=Bus::new(test::test_rom(vec![]), |ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);
        cpu.register_a = 10;

        cpu.load_and_run(vec![0xaa, 0x00]);

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let bus=Bus::new(test::test_rom(vec![]),|ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);

        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let bus=Bus::new(test::test_rom(vec![]),|ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);
        cpu.register_x = 0xff;

        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_lda_from_memory() {
        let bus=Bus::new(test::test_rom(vec![]),|ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_0x4a_lsr_accumulator() {
        let bus=Bus::new(test::test_rom(vec![]),|ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);

        cpu.load_and_run(vec![0xa9, 0x02, 0x4a, 0x00]);

        assert_eq!(cpu.register_a, 0x01);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_0x08_php_pushes_status_to_stack() {
        let bus=Bus::new(test::test_rom(vec![]),|ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);
        cpu.status.insert(CpuFlags::CARRY);
        let expected = (cpu.status | CpuFlags::BREAK | CpuFlags::BREAK2).bits();

        cpu.load_and_run(vec![0x08, 0x00]);

        assert_eq!(cpu.stack_pointer, STACK_RESET.wrapping_sub(1));
        assert_eq!(
            cpu.mem_read((STACK as u16) + STACK_RESET as u16),
            expected
        );
    }

    #[test]
    fn test_0x70_bvs_branch_taken() {
        let bus=Bus::new(test::test_rom(vec![]),|ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);
        cpu.status.insert(CpuFlags::OVERFLOW);

        cpu.load_and_run(vec![0x70, 0x02, 0xa9, 0x01, 0xa9, 0x05, 0x00]);

        assert_eq!(cpu.register_a, 0x05);
    }

// @trace-pilot a1e3416f02c5121a2e205c78e8e2e8bc862c29cf
    #[test]
    fn test_dex_loop_with_cpx_and_bne() {
        let bus=Bus::new(test::test_rom(vec![]),|ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);

// @trace-pilot a1e3416f02c5121a2e205c78e8e2e8bc862c29cf
        cpu.load_and_run(vec![
            0xa2, 0x08,       // LDX #$08
            0xca,             // decrement: DEX
            0x8e, 0x00, 0x02, // STX $0200
            0xe0, 0x03,       // CPX #$03
            0xd0, 0xf8,       // BNE decrement
            0x8e, 0x01, 0x02, // STX $0201
            0x00,             // BRK
        ]);

        assert_eq!(cpu.register_x, 0x03);
        assert_eq!(cpu.mem_read(0x0200), 0x03);
        assert_eq!(cpu.mem_read(0x0201), 0x03);
    }

// @trace-pilot cb666341ff39b91336b8a3d746e528a20513f7db
    #[test]
    fn test_stack_roundtrip_loop_with_absolute_y_store() {
        let bus=Bus::new(test::test_rom(vec![]),|ppu: &NesPPU|{});
        let mut cpu = CPU::new(bus);

// @trace-pilot cb666341ff39b91336b8a3d746e528a20513f7db
        cpu.load_and_run(vec![
            0xa2, 0x00,       // LDX #$00
            0xa0, 0x00,       // LDY #$00
            0x8a,             // firstloop: TXA
            0x99, 0x00, 0x02, // STA $0200,Y
            0x48,             // PHA
            0xe8,             // INX
            0xc8,             // INY
            0xc0, 0x10,       // CPY #$10
            0xd0, 0xf5,       // BNE firstloop
            0x68,             // secondloop: PLA
            0x99, 0x00, 0x02, // STA $0200,Y
            0xc8,             // INY
            0xc0, 0x20,       // CPY #$20
            0xd0, 0xf7,       // BNE secondloop
            0x00,             // BRK
        ]);

        for i in 0..0x10 {
            assert_eq!(cpu.mem_read(0x0200 + i), i as u8);
        }

        for i in 0..0x10 {
            assert_eq!(cpu.mem_read(0x0210 + i), (0x0f - i) as u8);
        }
    }
}
