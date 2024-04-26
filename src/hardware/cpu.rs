use {
	crate::{
		ascii::ascii,
		hardware::{
			hardware::{Hardware, HardwareSpecs},
			interrupt_controller::InterruptController,
			imp::clock_listener::ClockListener,
			mmu::Mmu
		}
	},
	tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver},
	std::io::{self, Write}
};

/**Just a bunch of match expressions.*/
pub struct Cpu {
	pub specs: HardwareSpecs,
	interrupt_controller: InterruptController,
	pub mmu: Mmu,
	pub cpu_clock_counter: u128,//this increments after each instruction, so it's not realistic
	pub pc: u16,
	ir: Opcode,
	a: u8,
	x: u8,
	y: u8,
	pub nv_bdizc: u8,
	pub buf: u8,
	pub pipeline_step: u8,
	pub mar: u16,
	pub mdr: u8
}

impl Hardware for Cpu {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	fn get_specs_mut(&mut self) -> &mut HardwareSpecs {return &mut self.specs;}
}

impl ClockListener for Cpu {
	/**Fetch, decode, execute, write back, and interrupt check all handled in this massive match expression.*/
	fn pulse(&mut self) {
		self.log(format!("CPU clock count: {}, Pipeline Step: 0x{:02X}, PC: 0x{:04X}, IR: {:?}, Acc: 0x{:02X}, X: 0x{:02X}, Y: 0x{:02X}, Status: 0b{:08b}",
		    self.cpu_clock_counter,
			self.pipeline_step,
			self.pc,
			self.ir,
			self.a,
			self.x,
			self.y,
			self.nv_bdizc
		).as_str());
		self.cpu_clock_counter += 1;
		match self.pipeline_step {
			0x00 => {self.fetch();}
			0x01 => {self.pipeline_step += 1;}//waiting for read
			0x02 => {
				self.check_read_ready();
				self.ir = Opcode::from(self.mdr).expect(format!("Received an invalid opcode 0x{:02X} at address 0x{:04X}", self.mdr, self.pc - 1).as_str());
				self.pipeline_step += 1;
			}
			0x03 => {
				//instructions with no operand can be executed immediately here
				//instructions with 1 or 2 operands should set the pipeline_step to be able to fetch the operands
				match self.ir {
					Opcode::TXA => {
						self.a = self.x;
						self.set_zero(self.a);
						self.set_negative(self.a);
						self.pipeline_step = 0x0A;
					}
					Opcode::TYA => {
						self.a = self.y;
						self.set_zero(self.a);
						self.set_negative(self.a);
						self.pipeline_step = 0x0A;
					}
					Opcode::TAX => {
						self.x = self.a;
						self.set_zero(self.x);
						self.set_negative(self.x);
						self.pipeline_step = 0x0A;
					}
					Opcode::TAY => {
						self.y = self.a;
						self.set_zero(self.y);
						self.set_negative(self.y);
						self.pipeline_step = 0x0A;
					}
					Opcode::NOP => {
						self.pipeline_step = 0x0A;
					}
					Opcode::BRK => {
						//I'm pretty sure the BRK actually takes 7 cycles because it messes with the stack
						self.set_break(true);
						self.set_interrupt(true);//doesn't check for an interrupt at the end of this instruction cycle
						self.pipeline_step = 0x0A;
					}
					Opcode::SYS => {
						match self.x {
							0x01 => {
								print!("{}", self.y);
								io::stdout().flush().expect("Could not flush output buffer");
								self.pipeline_step = 0x0A;
							}
							0x02 => {
								self.mar = self.pc.wrapping_add(self.y as u16);
								self.pipeline_step += 1;
								self.mmu.read(self.mar);
							}
							0x03 => {self.fetch();}
							_ => {panic!("Invalid SYS call. Wrong value in the X register.");}
						}
					}
					Opcode::LDAi | Opcode::LDXi | Opcode::LDYi | Opcode::BNEr | Opcode::LDAa | Opcode::STAa | Opcode::ADCa | Opcode::LDXa | Opcode::LDYa | Opcode::CPXa | Opcode::INCa => {
						self.fetch();
					}
				}
			}
			0x04 => {self.pipeline_step += 1;}//waiting for read
			0x05 => {
				self.check_read_ready();
				match self.ir {
					Opcode::LDAi => {
						self.a = self.mdr;
						self.set_zero(self.a);
						self.set_negative(self.a);
						self.pipeline_step = 0x0A;
					}
					Opcode::LDXi => {
						self.x = self.mdr;
						self.set_zero(self.x);
						self.set_negative(self.x);
						self.pipeline_step = 0x0A;
					}
					Opcode::LDYi => {
						self.y = self.mdr;
						self.set_zero(self.y);
						self.set_negative(self.y);
						self.pipeline_step = 0x0A;
					}
					Opcode::BNEr => {
						if self.nv_bdizc & Self::ZERO_FLAG == 0 {
							self.pc = (self.pc as i16).wrapping_add(self.mdr as i8 as i16) as u16;
						}
						self.pipeline_step = 0x0A;
					}
					Opcode::LDAa | Opcode::STAa | Opcode::ADCa | Opcode::LDXa | Opcode::LDYa | Opcode::CPXa | Opcode::INCa => {
						self.buf = self.mdr;
						self.fetch();
					}
					Opcode::SYS => {
						if self.x == 3 {
							self.buf = self.mdr;
							self.fetch();
						} else {
							print!("{}", ascii::ENCODER.get(&self.mdr).unwrap_or(&'\0'));
							io::stdout().flush().expect("Could not flush output buffer");
							self.pipeline_step = 0x0A;
						}
					}
					_ => {panic!("This code should be unreachable.");}
				}
			}
			0x06 => {self.pipeline_step += 1;}//waiting for read
			0x07 => {
				self.check_read_ready();
				self.set_mar_low(self.buf);
				self.set_mar_high(self.mdr);
				if self.ir == Opcode::STAa {self.mdr = self.a;}
				self.pipeline_step += 1;
				match self.ir {
					Opcode::LDAa | Opcode::ADCa | Opcode::LDXa | Opcode::LDYa | Opcode::CPXa | Opcode::INCa | Opcode::SYS => {
						self.mmu.read(self.mar);
					}
					Opcode::STAa => {
						self.mmu.write(self.mar, self.mdr);
					}
					_ => {panic!("This code should be unreachable.");}
				}
			}
			0x08 => {
				match self.ir {
					Opcode::LDAa | Opcode::ADCa | Opcode::LDXa | Opcode::LDYa | Opcode::CPXa | Opcode::INCa | Opcode::SYS => {
						//waiting for read
						self.pipeline_step += 1;
					}
					Opcode::STAa => {self.pipeline_step = 0x0A;}//waiting for read
					_ => {panic!("This code should be unreachable.");}
				}
			}
			0x09 => {
				self.check_read_ready();
				match self.ir {
					Opcode::LDAa => {
						self.a = self.mdr;
						self.set_zero(self.a);
						self.set_negative(self.a);
						self.pipeline_step = 0x0A;
					}
					Opcode::ADCa => {
						let b: u8 = self.mdr;
						let (result, overflow) = self.a.overflowing_add(b);
						self.set_zero(result);
						self.set_negative(result);
						self.set_carry(result <= self.a && b != 0);
						self.set_overflow(overflow);
						self.a = result;
						self.pipeline_step = 0x0A;
					}
					Opcode::LDXa => {
						self.x = self.mdr;
						self.set_zero(self.x);
						self.set_negative(self.x);
						self.pipeline_step = 0x0A;
					}
					Opcode::LDYa => {
						self.y = self.mdr;
						self.set_zero(self.y);
						self.set_negative(self.y);
						self.pipeline_step = 0x0A;
					}
					Opcode::CPXa => {
						let value: u8 = self.mdr;
						let difference: u8 = self.x.wrapping_sub(value);
						self.set_zero(difference);
						self.set_negative(difference);
						self.set_carry(self.x >= value);
						self.pipeline_step = 0x0A;
					}
					Opcode::INCa => {
						let mut value: u8 = self.mdr;
						value = value.wrapping_add(1);
						self.set_zero(value);
						self.set_negative(value);
						self.mdr = value;
						self.pipeline_step += 1;
						self.mmu.write(self.mar, self.mdr);
					}
					Opcode::SYS => {
						if self.mdr != 0 {
							print!("{}", ascii::ENCODER.get(&self.mdr).unwrap_or(&'\0'));
							io::stdout().flush().expect("Could not flush output buffer");
							self.mar = self.mar.wrapping_add(1);
							self.pipeline_step = 0x08;
							self.mmu.read(self.mar);
						} else {
							self.pipeline_step = 0x0A;
						}
					}
					_ => {panic!("This code should be unreachable.");}
				}
			}
			0x0A => {
				//waiting for write and/or checking for interrupt
				self.pipeline_step = 0x00;
				if self.nv_bdizc & Self::INTERRUPT_FLAG == 0 {
					let mut recv: bool = true;
					while recv {
						match self.interrupt_controller.io_rx.try_recv() {
							Ok(specs) => {self.interrupt_controller.priority_queue.push(specs);}
							Err(_) => {recv = false;}
						}
					}
					if let Some(event) = self.interrupt_controller.priority_queue.pop() {
						if let Some(interrupt) = self.interrupt_controller.io_devices.get(&event.iqr) {
							if let Some(c) = ascii::ENCODER.get(&interrupt.get_out_buf()) {
								print!("{}", c);
								io::stdout().flush().expect("Could not flush output buffer");
							}
						} else {
							panic!("Could not find I/O device Name: {}, IQR: {}", event.name, event.iqr);
						}
					}
				}
			}
			_ => {panic!("This code should be unreachable.");}
		}
	}
}

impl Cpu {
	const NEGATIVE_FLAG: u8 = 0b1000_0000;
	const OVERFLOW_FLAG: u8 = 0b0100_0000;
	pub const BREAK_FLAG: u8 = 0b0001_0000;
	const INTERRUPT_FLAG: u8 = 0b0000_0100;
	const ZERO_FLAG: u8 = 0b0000_0010;
	const CARRY_FLAG: u8 = 0b0000_0001;
	
	pub fn new(mem_tx: UnboundedSender<(u16, u8, bool)>, mem_rx: UnboundedReceiver<u8>) -> Self {
		let cpu: Self = Self {
			specs: HardwareSpecs::new("Cpu"),
			interrupt_controller: InterruptController::new(),
			mmu: Mmu::new(mem_tx, mem_rx),
			cpu_clock_counter: 0,
			pc: 0xfffc,//0xfffc/d is the reset vector
			ir: Opcode::BRK,
			a: 0x00,
			x: 0x00,
			y: 0x00,
			nv_bdizc: 0b00100000,
			buf: 0x00,
			pipeline_step: 0x00,
			mar: 0x0000,
			mdr: 0x00
		};
		//cpu.mmu.memory.set_references(&mut cpu.mar, &mut cpu.mdr, &cpu.pipeline_step);
		cpu.log("Created");
		return cpu;
	}
	
	/**If the Memory has finished performing the read operation, this function "opens the bus lines" and sets the MDR to the value that was read. Otherwise, it does nothing.*/
	pub fn check_read_ready(&mut self) {
		if let Ok(mdr) = self.mmu.rx.try_recv() {self.mdr = mdr;}
	}
	
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
			self.nv_bdizc |= Self::BREAK_FLAG;
			self.nv_bdizc &= !Self::INTERRUPT_FLAG;
		} else {
			self.nv_bdizc &= !Self::BREAK_FLAG;
			self.nv_bdizc |= Self::INTERRUPT_FLAG;
		}
	}
	fn set_interrupt(&mut self, set: bool) {
		if set {
			self.nv_bdizc |= Self::INTERRUPT_FLAG;
		} else {
			self.nv_bdizc &= !Self::INTERRUPT_FLAG;
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
	
	/**Loads the PC into the MAR, increments the pipeline_step, tells the MMU to request a read operation in memory, and increments the PC.*/
	fn fetch(&mut self) {
		self.mar = self.pc;
		self.pipeline_step += 1;
		self.mmu.read(self.mar);
		self.pc = self.pc.wrapping_add(1);
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
	fn from(opcode: u8) -> Option<Self> {
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