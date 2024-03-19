use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	memory::Memory,
	imp::clock_listener::ClockListener,
};

pub struct Cpu {
	pub specs: HardwareSpecs,
	pub memory: Memory,
	cpu_clock_counter: u128,
	pub PC: u16,
	S: u8,//points to 0x0100 + S
	A: u8,
	X: u8,
	Y: u8,
	pub NV_BDIZC: u8
}

impl Hardware for Cpu {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let cpu: Self = Self {
			specs: HardwareSpecs::new_default("Cpu"),
			memory: Memory::new(),
			cpu_clock_counter: 0,
			PC: 0xfffc,//0xfffc and 0xfffd hold the address that the program starts at
			S: 0xfd,//stack grows down
			A: 0x00,
			X: 0x00,
			Y: 0x00,
			NV_BDIZC: 0b00100000
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
	const NEGATIVE_FLAG: u8 = 0b1000_0000;
	const OVERFLOW_FLAG: u8 = 0b0100_0000;
	pub const BREAK_FLAG: u8 = 0b0001_0000;
	const ZERO_FLAG: u8 = 0b0000_0010;
	const CARRY_FLAG: u8 = 0b0000_0001;
	
	pub fn fetch(&mut self) -> u8 {
		let op: u8 = self.memory.get(self.PC);
		self.PC = self.PC.wrapping_add(1);
		return op;
	}
	
	fn set_negative(&mut self, n: u8) {
		if n & 0b10000000 != 0 {
			self.NV_BDIZC |= Self::NEGATIVE_FLAG;
		} else {
			self.NV_BDIZC &= !Self::NEGATIVE_FLAG;
		}
	}
	fn set_overflow(&mut self, v: bool) {
		if v {
			self.NV_BDIZC |= Self::OVERFLOW_FLAG;
		} else {
			self.NV_BDIZC &= !Self::OVERFLOW_FLAG;
		}
	}
	fn set_break(&mut self, set: bool) {
		if set {
			self.NV_BDIZC |= Self::BREAK_FLAG;
		} else {
			self.NV_BDIZC &= !Self::BREAK_FLAG;
		}
	}
	fn set_zero(&mut self, n: u8) {
		if n == 0 {
			self.NV_BDIZC |= Self::ZERO_FLAG;
		} else {
			self.NV_BDIZC &= !Self::ZERO_FLAG;
		}
	}
	fn set_carry(&mut self, c: bool) {
		if c {
			self.NV_BDIZC |= Self::CARRY_FLAG;
		} else {
			self.NV_BDIZC &= !Self::CARRY_FLAG;
		}
	}
	
	//TODO go through all the instructions and check how they should update the status register
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
					let (result, overflow) = self.A.overflowing_add(b);
					self.set_zero(result);
					self.set_negative(result);
					self.set_carry(result <= self.A && b != 0);
					self.set_overflow(overflow);
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
				Opcode::BRK => {self.set_break(true);}
				Opcode::CPXa(i, ii) => {
					let value: u8 = self.memory.get(Self::little_endian_to_u16(i, ii));
					self.set_zero(self.X.wrapping_sub(value));
					self.set_negative(self.X.wrapping_sub(value));
					self.set_carry(self.X >= value);
				}
				Opcode::BNEr(i) => {
					//I'm not sure when the overflow wraps back to a previous address vs carries to the next page
					//this implementation might night be accurate
					//Right now it tries to perform signed addition on the program counter
					if self.NV_BDIZC & Self::ZERO_FLAG == 0 {
						self.PC = (self.PC as i16).wrapping_add(i as i8 as i16) as u16;
					}
				}
				Opcode::INCa(i, ii) => {
					let addr: u16 = Self::little_endian_to_u16(i, ii);
					let mut value: u8 = self.memory.get(addr);
					value = value.wrapping_add(1);
					self.set_zero(value);
					self.set_negative(value);
					self.memory.set(addr, value);
				}
				Opcode::SYS => {
					match self.X {
						0x01 => {print!("{}", self.Y);}
						0x02 => {print!("{}", self.memory.get(self.PC + self.Y as u16) as char);}
						0x03 => {
							let mut address: u16 = Self::little_endian_to_u16(self.fetch(), self.fetch());
							let mut string: String = String::from("");
							while self.memory.get(address) != 0x00 {
								string.push(self.memory.get(address) as char);
								address = address.wrapping_add(1);
							}
							print!("{}", string);
						}
						_ => {}
					}
				}
			}
		} else {
			//The program will panic, so this will never execute
			//panic!("Received an invalid opcode");
		}
	}
}

/**6502 ASM opcodes.
Lowercase 4th letter indicates addressing mode.
i = immediate, a = absolute, r = relative.
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
			_ => {
				panic!("Received an invalid opcode 0x{:02X} at address 0x{:04X}", value, cpu.PC - 1);
				//None  //if this returns none, it needs to be handled in the calling function, not otherwise
			}
		}
	}
}