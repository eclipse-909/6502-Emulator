use crate::{
	hardware::{
		hardware::{Hardware, HardwareSpecs},
		memory::Memory,
		imp::clock_listener::ClockListener,
	},
	PRINT_STR_ADDR
};

pub struct Cpu {
	pub specs: HardwareSpecs,
	pub running: bool,
	pub memory: Box<Memory>,//too big to store 0x10000 bytes of memory on the stack
	cpu_clock_counter: u128,
	pub PC: u16,
	S: u8,//points to 0x0100 + S
	A: u8,
	X: u8,
	Y: u8,
	NV_BDIZC: u8
}

impl Hardware for Cpu {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let cpu: Self = Self {
			specs: HardwareSpecs::new_default("Cpu"),
			running: false,
			memory: Box::new(Memory::new()),
			cpu_clock_counter: 0,
			PC: 0xfffc,//0xfffc and 0xfffd hold the address that the program starts at
			S: 0xfd,//stack grows down
			A: 0x00,
			X: 0x00,
			Y: 0x00,
			NV_BDIZC: 0b00110001
		};
		cpu.log("Created");
		return cpu;
	}
}

impl ClockListener for Cpu {
	fn pulse(&mut self) {
		self.log(format!("Received clock pulse - CPU clock count: {}", self.cpu_clock_counter).as_str());
		self.cpu_clock_counter += 1;
		self.fetch_decode_execute();
		self.memory.pulse();
	}
}

impl Cpu {
	const CARRY_FLAG: u8 = 0b0000_0001;
	const ZERO_FLAG: u8 = 0b0000_0010;
	const OVERFLOW_FLAG: u8 = 0b0100_0000;
	const NEGATIVE_FLAG: u8 = 0b1000_0000;
	
	pub fn fetch(&mut self) -> u8 {
		let op: u8 = self.memory.get(self.PC);
		self.PC += 1;
		return op;
	}
	
	fn set_zero(&mut self, n: u8) {
		if n == 0 {
			self.NV_BDIZC |= Self::ZERO_FLAG;
		} else {
			self.NV_BDIZC &= !Self::ZERO_FLAG;
		}
	}
	
	fn set_negative(&mut self, n: u8) {
		if n & 0x80 != 0 {
			self.NV_BDIZC |= Self::NEGATIVE_FLAG;
		} else {
			self.NV_BDIZC &= !Self::NEGATIVE_FLAG;
		}
	}
	
	fn set_carry(&mut self, c: bool) {
		if c {
			self.NV_BDIZC |= Self::CARRY_FLAG;
		} else {
			self.NV_BDIZC &= !Self::CARRY_FLAG;
		}
	}
	
	fn fetch_decode_execute(&mut self) {
		if let Some(opcode) = Opcode::fetch_decode(self) {
			match opcode {
				Opcode::LDAi(i) => {
					self.A = i;
					self.set_zero(self.A);
					self.set_negative(self.A);
				}
				Opcode::LDAa(i, ii) => {
					self.A = self.memory.get(Self::little_endian_to_u16(i, ii));
					self.set_zero(self.A);
					self.set_negative(self.A);
				}
				Opcode::STAa(i, ii) => {self.memory.set(Self::little_endian_to_u16(i, ii), self.A);}
				Opcode::TXA => {
					self.A = self.X;
					self.set_zero(self.A);
					self.set_negative(self.A);
				}
				Opcode::TYA => {
					self.A = self.Y;
					self.set_zero(self.A);
					self.set_negative(self.A);
				}
				Opcode::ADCa(i, ii) => {
					let b: u8 = self.memory.get(Self::little_endian_to_u16(i, ii));
					let result = self.A.wrapping_add(b).wrapping_add(if self.NV_BDIZC & Self::CARRY_FLAG != 0 { 1 } else { 0 });
					self.set_zero(result);
					self.set_negative(result);
					self.set_carry(result < self.A);
					if (self.A ^ b) & 0x80 == 0 && (self.A ^ result) & 0x80 != 0 {
						self.NV_BDIZC |= Self::OVERFLOW_FLAG;
					} else {
						self.NV_BDIZC &= !Self::OVERFLOW_FLAG;
					}
					self.A = result;
				}
				Opcode::LDXi(i) => {
					self.X = i;
					self.set_zero(self.X);
					self.set_negative(self.X);
				}
				Opcode::LDXa(i, ii) => {
					self.X = self.memory.get(Self::little_endian_to_u16(i, ii));
					self.set_zero(self.X);
					self.set_negative(self.X);
				}
				Opcode::TAX => {
					self.X = self.A;
					self.set_zero(self.X);
					self.set_negative(self.X);
				}
				Opcode::LDYi(i) => {
					self.Y = i;
					self.set_zero(self.Y);
					self.set_negative(self.Y);
				}
				Opcode::LDYa(i, ii) => {
					self.Y = self.memory.get(Self::little_endian_to_u16(i,  ii));
					self.set_zero(self.Y);
					self.set_negative(self.Y);
				}
				Opcode::TAY => {
					self.Y = self.A;
					self.set_zero(self.Y);
					self.set_negative(self.Y);
				}
				Opcode::NOP => {}
				Opcode::BRK => {self.running = false; self.NV_BDIZC |= 0b1_0100;}
				Opcode::CPXa(i, ii) => {
					let value: u8 = self.memory.get(Self::little_endian_to_u16(i, ii));
					self.set_zero(self.X);
					self.set_negative(self.X);
					self.set_carry(self.X >= value);
				}
				Opcode::BNEr(i) => {if self.NV_BDIZC & Self::ZERO_FLAG == 0 {self.PC += i as u16;}}
				Opcode::INCa(i, ii) => {
					let addr: u16 = Self::little_endian_to_u16(i, ii);
					let value: u8 = self.memory.get(addr);
					self.set_zero(value);
					self.set_negative(value);
					self.memory.set(addr, value + 1);
				}
				Opcode::SYS => {
					match self.X {
						0x01 => {print!("{}", self.Y);}
						0x02 => {print!("{}", self.Y as char);}
						0x03 => {
							let &[i, ii] = &[self.fetch(), self.fetch()];
							self.memory.set_range(0x00, &[i, ii]);
							let curr_addr: (u8, u8) = Self::u16_to_little_endian(&self.PC);
							self.memory.set_range(0x0100 + self.S as u16, &[curr_addr.0, curr_addr.1]);
							self.S -= 2;
							self.PC = PRINT_STR_ADDR;
						}
						_ => {}
					}
				}
				
				Opcode::JSRa(i, ii) => {
					let curr_addr: (u8, u8) = Self::u16_to_little_endian(&self.PC);
					self.memory.set_range(0x0100 + self.S as u16, &[curr_addr.0, curr_addr.1]);
					self.S -= 2;
					self.PC = Self::little_endian_to_u16(i, ii);
				}
				Opcode::RTS => {
					self.S += 2;
					let addr: &[u8] = self.memory.get_range(0x0100 + self.S as u16, 2);
					self.PC = Self::little_endian_to_u16(addr[0], addr[1]);
				}
				Opcode::LDAn(i) => {
					self.A = self.memory.get(Self::little_endian_to_u16(self.memory.get(i as u16), self.memory.get(i as u16 + 1)) + self.Y as u16);
					self.set_zero(self.A);
					self.set_negative(self.A);
				}
				Opcode::BEQr(i) => {
					if self.NV_BDIZC & Self::ZERO_FLAG != 0 {
						self.PC += i as u16;
					}
				}
				Opcode::INX => {
					self.X += 1;
					self.set_zero(self.X);
					self.set_negative(self.X);
				}
				Opcode::JMPa(i, ii) => {self.PC = Self::little_endian_to_u16(i, ii);}
			}
		} else {
			//I'll get around to handling this
			panic!("Received an invalid opcode");
		}
	}
}

/**6502 ASM opcodes.
Lowercase 4th letter indicates addressing mode.
i = immediate, a = absolute, n = indirect.
Parameters indicate operands.*/
#[repr(u8)]
#[derive(Debug)]
enum Opcode {
	LDAi(u8)        = 0xA9,     //load immediate u8 into A
	LDAa(u8, u8)    = 0xAD,     //load value from memory into A
	STAa(u8, u8)    = 0x8D,     //store A into memory
	TXA             = 0x8A,     //transfer X to A
	TYA             = 0x98,     //transfer Y to A
	ADCa(u8, u8)    = 0x6D,     //add value from memory to A
	LDXi(u8)        = 0xA2,     //load immediate u8 into X
	LDXa(u8, u8)    = 0xAE,     //load value from memory into X
	TAX             = 0xAA,     //transfer A to X
	LDYi(u8)        = 0xA0,     //load immediate u8 into Y
	LDYa(u8, u8)    = 0xAC,     //load value from memory into Y
	TAY             = 0xA8,     //transfer A to Y
	NOP             = 0xEA,     //no operation
	BRK             = 0x00,     //break
	CPXa(u8, u8)    = 0xEC,     //compare X with value from memory
	BNEr(u8)        = 0xD0,     //branch if zero-flag != 0
	INCa(u8, u8)    = 0xEE,     //increment A
	SYS             = 0xFF,     //syscall may have operands, but that must be handled in impl Cpu::fetch_decode_execute
	
	JSRa(u8, u8)     = 0x20,     //jump to new address, pushing current address onto stack
	RTS             = 0x60,     //pop the address from the top of the stack and go there
	LDAn(u8)        = 0xB1,     //load into the accumulator the value pointed to by the address stored in the zero-page based address with the given operand, and indexed by the value in the Y register
	BEQr(u8)         = 0xF0,     //branch if Z = 1 to the relative address
	INX             = 0xE8,     //increment X
	JMPa(u8, u8)     = 0x4C      //jump to the given address
}

impl Opcode {
	fn fetch_decode(cpu: &mut Cpu) -> Option<Opcode> {
		let value: u8 = cpu.fetch();
		return match value {
			0xA9 => {Some(Opcode::LDAi(cpu.fetch()))}
			0xAD => {Some(Opcode::LDAa(cpu.fetch(), cpu.fetch()))}
			0x8D => {Some(Opcode::STAa(cpu.fetch(), cpu.fetch()))}
			0x8A => {Some(Opcode::TXA)}
			0x98 => {Some(Opcode::TYA)}
			0x6D => {Some(Opcode::ADCa(cpu.fetch(), cpu.fetch()))}
			0xA2 => {Some(Opcode::LDXi(cpu.fetch()))}
			0xAE => {Some(Opcode::LDXa(cpu.fetch(), cpu.fetch()))}
			0xAA => {Some(Opcode::TAX)}
			0xA0 => {Some(Opcode::LDYi(cpu.fetch()))}
			0xAC => {Some(Opcode::LDYa(cpu.fetch(), cpu.fetch()))}
			0xA8 => {Some(Opcode::TAY)}
			0xEA => {Some(Opcode::NOP)}
			0x00 => {Some(Opcode::BRK)}
			0xEC => {Some(Opcode::CPXa(cpu.fetch(), cpu.fetch()))}
			0xD0 => {Some(Opcode::BNEr(cpu.fetch()))}
			0xEE => {Some(Opcode::INCa(cpu.fetch(), cpu.fetch()))}
			0xFF => {Some(Opcode::SYS)}
			
			0x20 => {Some(Opcode::JSRa(cpu.fetch(), cpu.fetch()))}
			0x60 => {Some(Opcode::RTS)}
			0xB1 => {Some(Opcode::LDAn(cpu.fetch()))}
			0xF0 => {Some(Opcode::BEQr(cpu.fetch()))}
			0xE8 => {Some(Opcode::INX)}
			0x4C => {Some(Opcode::JMPa(cpu.fetch(), cpu.fetch()))}
			_ => {None}//TODO print out the given opcode here and the error message
		}
	}
}