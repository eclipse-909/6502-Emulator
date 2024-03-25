use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener,
	mmu::MMU
};

pub struct Cpu {
	pub specs: HardwareSpecs,
	pub mmu: MMU,
	pub cpu_clock_counter: u128,//this increments after each instruction, so it's not realistic
	pub pc: u16,
	ir: Opcode,
	sp: u8,//points to 0x0100 + sp
	a: u8,
	x: u8,
	y: u8,
	pub nv_bdizc: u8,
	buf: u8,
	pipeline_step: u8,
	mar: u16,
	mdr: u8
}

impl Hardware for Cpu {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let cpu: Self = Self {
			specs: HardwareSpecs::new_default("Cpu"),
			mmu: MMU::new(),
			cpu_clock_counter: 0,
			pc: 0xfffc,//0xfffc and 0xfffd hold the address that the program starts at
			ir: Opcode::BRK,
			sp: 0xff,//stack grows down
			a: 0x00,
			x: 0x00,
			y: 0x00,
			nv_bdizc: 0b00100000,
			buf: 0x00,
			pipeline_step: 0x00,
			mar: 0x0000,
			mdr: 0x00
		};
		cpu.log("Created");
		return cpu;
	}
}

impl ClockListener for Cpu {
	/*TODO ask Prof. Gormanly about how his emulator sets the status register
	A9 - Load Accumulator Immediate: Does not affect the status register.
	AD - Load Accumulator from Memory (Absolute): Does not affect the status register.
	8D - Store Accumulator in Memory (Absolute): Does not affect the status register.
	8A - Transfer X to Accumulator: Updates the zero flag (Z) and the negative flag (N) based on the result.
	98 - Transfer Y to Accumulator: Updates the zero flag (Z) and the negative flag (N) based on the result.
	6D - Add with Carry (Absolute): Updates the zero flag (Z), negative flag (N), overflow flag (V), and carry flag (C) based on the result.
	A2 - Load X Immediate: Updates the zero flag (Z) and the negative flag (N) based on the result.
	AE - Load X from Memory (Absolute): Updates the zero flag (Z) and the negative flag (N) based on the result.
	AA - Transfer Accumulator to X: Updates the zero flag (Z) and the negative flag (N) based on the result.
	A0 - Load Y Immediate: Updates the zero flag (Z) and the negative flag (N) based on the result.
	AC - Load Y from Memory (Absolute): Updates the zero flag (Z) and the negative flag (N) based on the result.
	A8 - Transfer Accumulator to Y: Updates the zero flag (Z) and the negative flag (N) based on the result.
	EA - No Operation: Does not affect the status register.
	00 - Break: Updates the break flag (B) and the interrupt disable flag (I).
	EC - Compare X with Memory (Absolute): Updates the zero flag (Z), negative flag (N), and carry flag (C) based on the comparison result.
	D0 - Branch if Not Equal: Does not affect the status register.
	EE - Increment Memory (Absolute): Updates the zero flag (Z) and the negative flag (N) based on the result.
	*/
	fn pulse(&mut self) {
		self.log(format!("Received clock pulse - CPU clock count: {}", self.cpu_clock_counter).as_str());
		self.cpu_clock_counter += 1;
		match self.pipeline_step {
			0x00 => {
				self.mar = self.pc;
				self.pipeline_step = 0x01;
			}
			0x01 => {
				self.mmu.memory.read(self.mar, &mut self.mdr);
				self.pipeline_step = 0x02;
			}
			0x02 => {
				self.ir = Opcode::from_u8(self.mdr).expect(format!("Received an invalid opcode 0x{:02X} at address 0x{:04X}", self.mdr, self.pc).as_str());
				self.pc = self.pc.wrapping_add(1);
				self.pipeline_step = 0x03;
			}
			0x03 => {
				//instructions with no operand can be executed immediately here
				//instructions with 1 or 2 operands should set the pipeline_step to be able to fetch the operand
				match self.ir {
					Opcode::TXA => {
						self.a = self.x;
						self.set_zero(self.a);
						self.set_negative(self.a);
						self.pipeline_step = 0x00;
					}
					Opcode::TYA => {
						self.a = self.y;
						self.set_zero(self.a);
						self.set_negative(self.a);
						self.pipeline_step = 0x00;
					}
					Opcode::TAX => {
						self.x = self.a;
						self.set_zero(self.x);
						self.set_negative(self.x);
						self.pipeline_step = 0x00;
					}
					Opcode::TAY => {
						self.y = self.a;
						self.set_zero(self.y);
						self.set_negative(self.y);
						self.pipeline_step = 0x00;
					}
					Opcode::NOP => {
						self.pipeline_step = 0x00;
					}
					Opcode::BRK => {
						//I'm pretty sure the BRK actually takes 7 cycles, but I don't feel like doing all that
						self.set_break(true);
						self.pipeline_step = 0x00;
					}
					Opcode::SYS => {
						match self.x {
							0x01 => {
								print!("{}", self.y);
								self.pipeline_step = 0x00;
							}
							0x02 => {
								self.mar = self.pc.wrapping_add(self.y as u16);
								self.pipeline_step = 0x04;
							}
							0x03 => {
								self.mar = self.pc;
								self.pc = self.pc.wrapping_add(1);
								self.pipeline_step = 0x04;
							}
							_ => {panic!("Invalid SYS call. Wrong value in the X register.");}
						}
					}
					Opcode::LDAi | Opcode::LDXi | Opcode::LDYi | Opcode::BNEr | Opcode::LDAa | Opcode::STAa | Opcode::ADCa | Opcode::LDXa | Opcode::LDYa | Opcode::CPXa | Opcode::INCa => {
						//set the pipeline_step to fetch 1 operand
						self.mar = self.pc;
						self.pc = self.pc.wrapping_add(1);
						self.pipeline_step = 0x04;
					}
				}
			}
			0x04 => {
				self.mmu.memory.read(self.mar, &mut self.mdr);
				self.pipeline_step = 0x05
			}
			0x05 => {
				match self.ir {
					Opcode::LDAi => {
						self.a = self.mdr;
						self.pipeline_step = 0x00;
					}
					Opcode::LDXi => {
						self.x = self.mdr;
						self.set_zero(self.x);
						self.set_negative(self.x);
						self.pipeline_step = 0x00;
					}
					Opcode::LDYi => {
						self.y = self.mdr;
						self.set_zero(self.y);
						self.set_negative(self.y);
						self.pipeline_step = 0x00;
					}
					Opcode::BNEr => {
						if self.nv_bdizc & Self::ZERO_FLAG == 0 {
							self.pc = (self.pc as i16).wrapping_add(self.mdr as i8 as i16) as u16;
						}
						self.pipeline_step = 0x00;
					}
					Opcode::LDAa | Opcode::STAa | Opcode::ADCa | Opcode::LDXa | Opcode::LDYa | Opcode::CPXa | Opcode::INCa => {
						self.buf = self.mdr;
						self.mar = self.pc;
						self.pc = self.pc.wrapping_add(1);
						self.pipeline_step = 0x06;
					}
					Opcode::SYS => {
						if self.x == 3 {
							self.buf = self.mdr;
							self.mar = self.pc;
							self.pc = self.pc.wrapping_add(1);
							self.pipeline_step = 0x06;
						} else {
							print!("{}", self.mdr as char);
							self.pipeline_step = 0x00;
						}
					}
					_ => {panic!("This code should be unreachable.");}
				}
			}
			0x06 => {
				self.mmu.memory.read(self.mar, &mut self.mdr);
				self.pipeline_step = 0x07;
			}
			0x07 => {
				self.set_mar_low(self.buf);
				self.set_mar_high(self.mdr);
				if self.ir == Opcode::STAa {self.mdr = self.a;}
				self.pipeline_step = 0x08;
			}
			0x08 => {
				match self.ir {
					Opcode::LDAa | Opcode::ADCa | Opcode::LDXa | Opcode::LDYa | Opcode::CPXa | Opcode::INCa | Opcode::SYS => {
						self.mmu.memory.read(self.mar, &mut self.mdr);
						self.pipeline_step = 0x09;
					}
					Opcode::STAa => {
						self.mmu.memory.write(self.mar, self.mdr);
						self.pipeline_step = 0x00;
					}
					_ => {panic!("This code should be unreachable.");}
				}
			}
			0x09 => {
				match self.ir {
					Opcode::LDAa => {
						self.a = self.mdr;
						self.pipeline_step = 0x00;
					}
					Opcode::ADCa => {
						let b: u8 = self.mdr;
						let (result, overflow) = self.a.overflowing_add(b);
						self.set_zero(result);
						self.set_negative(result);
						self.set_carry(result <= self.a && b != 0);
						self.set_overflow(overflow);
						self.a = result;
						self.pipeline_step = 0x00;
					}
					Opcode::LDXa => {
						self.x = self.mdr;
						self.set_zero(self.x);
						self.set_negative(self.x);
						self.pipeline_step = 0x00;
					}
					Opcode::LDYa => {
						self.y = self.mdr;
						self.set_zero(self.y);
						self.set_negative(self.y);
						self.pipeline_step = 0x00;
					}
					Opcode::CPXa => {
						let value: u8 = self.mdr;
						let difference: u8 = self.x.wrapping_sub(value);
						self.set_zero(difference);
						self.set_negative(difference);
						self.set_carry(self.x >= value);
						self.pipeline_step = 0x00;
					}
					Opcode::INCa => {
						let mut value: u8 = self.mdr;
						value = value.wrapping_add(1);
						self.set_zero(value);
						self.set_negative(value);
						self.mdr = value;
						self.pipeline_step = 0x0A;
					}
					Opcode::SYS => {
						if self.mdr != 0 {
							print!("{}", self.mdr as char);
							self.mar = self.mar.wrapping_add(1);
							self.pipeline_step = 0x08;
						} else {
							self.pipeline_step = 0x00;
						}
					}
					_ => {panic!("This code should be unreachable.");}
				}
			}
			0x0A => {
				self.mmu.memory.write(self.mar, self.mdr);
				self.pipeline_step = 0x00;
			}
			_ => {panic!("This code should be unreachable.");}
		}
	}
}

impl Cpu {
	const NEGATIVE_FLAG: u8 = 0b1000_0000;
	const OVERFLOW_FLAG: u8 = 0b0100_0000;
	pub const BREAK_AND_INTERRUPT_FLAG: u8 = 0b0001_0100;
	const ZERO_FLAG: u8 = 0b0000_0010;
	const CARRY_FLAG: u8 = 0b0000_0001;
	
	/**Sets the low byte of the MAR.*/
	pub fn set_mar_low(&mut self, low_byte: u8) {self.mar = (self.mar & 0xFF00) | (low_byte as u16);}
	/**Sets the high byte of the MAR.*/
	pub fn set_mar_high(&mut self, high_byte: u8) {self.mar = (self.mar & 0x00FF) | ((high_byte as u16) << 8);}
	
	fn set_negative(&mut self, n: u8) {
		if n & Self::NEGATIVE_FLAG == Self::NEGATIVE_FLAG {
			self.nv_bdizc |= Self::NEGATIVE_FLAG;
		} else {
			self.nv_bdizc &= !Self::NEGATIVE_FLAG;
		}
	}
	fn set_overflow(&mut self, v: bool) {
		if v {
			self.nv_bdizc |= Self::OVERFLOW_FLAG;
		} else {
			self.nv_bdizc &= !Self::OVERFLOW_FLAG;
		}
	}
	fn set_break(&mut self, set: bool) {
		if set {
			self.nv_bdizc |= Self::BREAK_AND_INTERRUPT_FLAG;
		} else {
			self.nv_bdizc &= !Self::BREAK_AND_INTERRUPT_FLAG;
		}
	}
	fn set_zero(&mut self, n: u8) {
		if n == 0 {
			self.nv_bdizc |= Self::ZERO_FLAG;
		} else {
			self.nv_bdizc &= !Self::ZERO_FLAG;
		}
	}
	fn set_carry(&mut self, c: bool) {
		if c {
			self.nv_bdizc |= Self::CARRY_FLAG;
		} else {
			self.nv_bdizc &= !Self::CARRY_FLAG;
		}
	}
}

/**6502 ASM opcodes.
Lowercase 4th letter indicates addressing mode.
i = immediate, a = absolute, r = relative, none = accumulator.
Parameters indicate operands.*/
#[repr(u8)]
#[derive(Debug, PartialEq)]
enum Opcode {
	LDAi = 0xA9,     //load immediate u8 into a
	LDAa = 0xAD,     //load value from memory into a
	STAa = 0x8D,     //store a into memory
	TXA  = 0x8A,     //transfer x to a
	TYA  = 0x98,     //transfer y to a
	ADCa = 0x6D,     //add value from memory to a
	LDXi = 0xA2,     //load immediate u8 into x
	LDXa = 0xAE,     //load value from memory into x
	TAX  = 0xAA,     //transfer a to x
	LDYi = 0xA0,     //load immediate u8 into y
	LDYa = 0xAC,     //load value from memory into y
	TAY  = 0xA8,     //transfer a to y
	NOP  = 0xEA,     //no operation
	BRK  = 0x00,     //break
	CPXa = 0xEC,     //compare x with value from memory
	BNEr = 0xD0,     //branch if zero-flag != 0
	INCa = 0xEE,     //increment a
	SYS  = 0xFF,     //syscall may have operands
}

impl Opcode {
	fn from_u8(opcode: u8) -> Option<Self> {
		return match opcode {
			0xA9 => {Some(Opcode::LDAi)}
			0xAD => {Some(Opcode::LDAa)}
			0x8D => {Some(Opcode::STAa)}
			0x8A => {Some(Opcode::TXA)}
			0x98 => {Some(Opcode::TYA)}
			0x6D => {Some(Opcode::ADCa)}
			0xA2 => {Some(Opcode::LDXi)}
			0xAE => {Some(Opcode::LDXa)}
			0xAA => {Some(Opcode::TAX)}
			0xA0 => {Some(Opcode::LDYi)}
			0xAC => {Some(Opcode::LDYa)}
			0xA8 => {Some(Opcode::TAY)}
			0xEA => {Some(Opcode::NOP)}
			0x00 => {Some(Opcode::BRK)}
			0xEC => {Some(Opcode::CPXa)}
			0xD0 => {Some(Opcode::BNEr)}
			0xEE => {Some(Opcode::INCa)}
			0xFF => {Some(Opcode::SYS)}
			_ => {None}
		}
	}
}