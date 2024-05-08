use {
	crate::{
		ascii::ascii,
		hardware::{
			hardware::{Hardware, HardwareSpecs},
			interrupt_controller::InterruptController,
			imp::clock_listener::ClockListener,
			mmu::Mmu,
			memory::{MemEvent, N_WAYS}
		}
	},
	tokio::sync::mpsc::{Sender, Receiver},
	std::{
		io::{self, Write},
		cmp::PartialEq
	}
};

/**Just a bunch of match expressions.*/
pub struct Cpu {
	pub specs: HardwareSpecs,
	interrupt_controller: InterruptController,
	pub mmu: Mmu,
	pub cpu_clock_counter: u128,
	pub instruction_counter: u128,
	///Shared between fetch and decode, but execution units get their own instruction pointer
	pub pc: u16,
	///Set to Some to let fetch and decode know if they need to run. Operands are set to Some to tell decode if the instruction is ready to be executed
	ir: Option<(Opcode, Option<u8>, Option<u8>)>,
	a: u8,
	x: u8,
	y: u8,
	pub nv_bdizc: u8,
	execution_units: [ExecutionUnit; 2],
	pipe_mem_user: PipeMemUser
}

impl Hardware for Cpu {
	fn get_specs(&self) -> &HardwareSpecs {&self.specs}
}

impl ClockListener for Cpu {
	/**Fetch, decode, execute, write back, and interrupt check all handled in this massive match expression.*/
	fn pulse(&mut self) {
		self.log(format!("CPU clock count: {}, PC: 0x{:04X}, IR: {}, Acc: 0x{:02X}, X: 0x{:02X}, Y: 0x{:02X}, Status: 0b{:08b}",
		    self.cpu_clock_counter,
			self.pc,
			if let Some((opcode, _, _)) = self.ir.to_owned() {format!("{:?}", opcode)} else {String::from("Empty")},
			self.a,
			self.x,
			self.y,
			self.nv_bdizc
		).as_str());
		self.cpu_clock_counter += 1;
		/*Fetch, Decode, Execute are called in reverse order to prioritize memory access to the first function to be called.
		The pipeline is running like an assembly line. If the pipeline doesn't stall too much, it should be able to execute
		instructions faster than 1 instruction per instruction cycle, but probably under scalar speed.*/
		for i in 0..self.execution_units.len() {
			self.execute(i);
		}
		self.decode();
		self.fetch_opcode();
		//Allow memory access if nobody needs it in the next cycle
		if self.pipe_mem_user == PipeMemUser::Complete {
			self.pipe_mem_user = PipeMemUser::Free;
			self.interrupt_check();//I put this here just so it does an interrupt check a couple of times per instruction cycle rather than every clock cycle
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
	
	pub fn new(channels: [(Sender<MemEvent>, Receiver<MemEvent>); N_WAYS as usize]) -> Self {
		let cpu: Self = Self {
			specs: HardwareSpecs::new("Cpu"),
			interrupt_controller: InterruptController::new(),
			mmu: Mmu::new(channels),
			cpu_clock_counter: 0,
			instruction_counter: 0,
			pc: 0x0000,
			ir: None,
			a: 0x00,
			x: 0x00,
			y: 0x00,
			nv_bdizc: 0b00100000,
			execution_units: [ExecutionUnit::new(0), ExecutionUnit::new(1)],
			pipe_mem_user: PipeMemUser::Free
		};
		cpu.log("Created");
		cpu
	}
	
	fn sys_out_char(c: char) {
		print!("{}", c);
		io::stdout().flush().expect("Could not flush output buffer");//This prints each character one at a time immediately, rather than printing a buffer of many characters
	}
	
	fn sys_out_u8(n: u8) {
		print!("{:X}", n);
		io::stdout().flush().expect("Could not flush output buffer");
	}
	
	///Sets the pipe_mem_user and returns Cache::read(addr)
	fn read(&mut self, addr: u16, user: PipeMemUser) -> Result<Option<u8>,()> {
		self.pipe_mem_user = user;
		self.mmu.cache.read(addr)
	}
	///Sets the pipe_mem_user and returns Cache::write(addr, value)
	fn write(&mut self, addr: u16, value: u8, user: PipeMemUser) -> bool {
		self.pipe_mem_user = user;
		self.mmu.cache.write(addr, value)
	}
	
	//functions to set the status register bit flags
	fn set_negative(&mut self, n: u8) {
		if n & Self::NEGATIVE_FLAG == Self::NEGATIVE_FLAG {
			self.nv_bdizc |= Self::NEGATIVE_FLAG;
		} else {
			self.nv_bdizc &= !Self::NEGATIVE_FLAG;
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
	
	/**Clears all buffers if there's a BRK or branch to prevent executing more instructions.
	There's no branch speculation. So the pipeline is cleared on a branch, and it continues as normal otherwise.*/
	pub fn clear_pipeline(&mut self) {
		self.ir = None;
		self.execution_units.iter_mut().for_each(|exe| {exe.busy = false;});
	}
	
	///Loads the PC into the MAR, increments the pipeline_step, tells the MMU to request a read operation in memory, and increments the PC.
	fn fetch_opcode(&mut self) {
		if self.ir.is_some() {return;}
		match self.pipe_mem_user {
			PipeMemUser::Fetch | PipeMemUser::Free => {
				if let Ok(Some(num)) = self.read(self.pc, PipeMemUser::Fetch) {
					let opcode: Opcode = Opcode::from(num).expect("Received invalid opcode.");
					self.ir = Some((opcode, None, None));
					self.pc = self.pc.wrapping_add(1);
					self.pipe_mem_user = PipeMemUser::Complete;
				}
			}
			_ => {}
		}
	}
	///Like fetch_opcode() but the value is returned instead of loaded into the IR
	fn fetch_operand(&mut self) -> Option<u8> {
		if self.ir.is_none() {return None;}
		match self.pipe_mem_user {
			PipeMemUser::Decode | PipeMemUser::Free => {
				if let Ok(Some(num)) = self.read(self.pc, PipeMemUser::Decode) {
					self.pc = self.pc.wrapping_add(1);
					self.pipe_mem_user = PipeMemUser::Complete;
					return Some(num);
				}
			}
			_ => {}
		}
		None
	}
	///Decodes the value in the IR and loads it into an available execution unit if finished decoding
	fn decode(&mut self) {
		if self.ir.is_none() {return;}
		let Some((opcode, mut operand1, mut operand2)) = self.ir.to_owned() else {return;};
		//see which storage areas will be affected in the next cycle
		let mut affected_storages: Vec<Storage> = Vec::new();
		self.execution_units.iter_mut().filter(|exe| {exe.busy}).for_each(|exe| {
			affected_storages.append(&mut exe.ir.0.affected_storage());
		});
		//get operands
		match opcode {
			Opcode::TXA | Opcode::TYA | Opcode::TAX | Opcode::TAY | Opcode::NOP | Opcode::BRK => {
				operand1 = Some(0x00);
				operand2 = Some(0x00);
			}
			Opcode::LDAi | Opcode::LDXi | Opcode::LDYi | Opcode::BNEr => {
				if operand1.is_none() {
					operand1 = self.fetch_operand();
				}
				operand2 = Some(0x00);
			}
			Opcode::LDAa | Opcode::STAa | Opcode::ADCa | Opcode::LDXa | Opcode::LDYa | Opcode::CPXa | Opcode::INCa => {
				if operand1.is_none() {
					operand1 = self.fetch_operand();
				} else if operand2.is_none() {
					operand2 = self.fetch_operand();
				}
			}
			Opcode::SYS => {
				//We can't decode a SYS if the execution units will affect the X register
				if !affected_storages.contains(&Storage::X) {
					match self.x {
						0x01 | 0x02 => {
							operand1 = Some(0x00);
							operand2 = Some(0x00);
						}
						0x03 => {
							if operand1.is_none() {
								operand1 = self.fetch_operand();
							} else if operand2.is_none() {
								operand2 = self.fetch_operand();
							}
						}
						_ => {panic!("Invalid arguments for system call.");}
					}
				}
			}
		}
		self.ir = Some((opcode.clone(), operand1, operand2));
		//if the instruction is ready to be sent to an execution unit, and if the execution units are ready to take the instruction
		if !opcode.dependent_storage().iter().any(|storage| {affected_storages.contains(storage)}) {
			let Some(exe) = self.execution_units.iter_mut().find(|exe| {!exe.busy}) else {return;};
			if let Some((opcode, Some(operand1), Some(operand2))) = self.ir.to_owned() {
				exe.set_instruction(self.pc, (opcode, operand1, operand2));
				self.ir = None;
			}
		}
	}
	///Executes the instruction in the execution unit at the given index of the exe_units array
	fn execute(&mut self, exe_index: usize) {
		if !self.execution_units[exe_index].busy {return;}
		match self.execution_units[exe_index].ir.0 {
			Opcode::LDAi => {
				self.a = self.execution_units[exe_index].ir.1;
				self.set_zero(self.a);
				self.set_negative(self.a);
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			Opcode::TXA => {
				self.a = self.x;
				self.set_zero(self.a);
				self.set_negative(self.a);
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			Opcode::TYA => {
				self.a = self.y;
				self.set_zero(self.a);
				self.set_negative(self.a);
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			Opcode::LDXi => {
				self.x = self.execution_units[exe_index].ir.1;
				self.set_zero(self.x);
				self.set_negative(self.x);
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			Opcode::TAX => {
				self.x = self.a;
				self.set_zero(self.x);
				self.set_negative(self.x);
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			Opcode::LDYi => {
				self.y = self.execution_units[exe_index].ir.1;
				self.set_zero(self.y);
				self.set_negative(self.y);
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			Opcode::TAY => {
				self.y = self.a;
				self.set_zero(self.y);
				self.set_negative(self.y);
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			Opcode::NOP => {
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			Opcode::BRK => {
				//I'm pretty sure the BRK actually takes 7 cycles because it messes with the stack
				self.nv_bdizc |= Self::BREAK_FLAG;
				self.nv_bdizc |= Self::INTERRUPT_FLAG;//doesn't check for an interrupt at the end of this instruction cycle
				self.clear_pipeline();
				self.instruction_counter += 1;
			}
			Opcode::BNEr => {
				if self.nv_bdizc & Self::ZERO_FLAG == 0 {
					self.pc = (self.execution_units[exe_index].ip as i16).wrapping_add(self.execution_units[exe_index].ir.1 as i8 as i16) as u16;
					self.clear_pipeline();
				}
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			Opcode::SYS if self.x == 1 => {
				Self::sys_out_u8(self.y);
				self.execution_units[exe_index].busy = false;
				self.instruction_counter += 1;
			}
			_ => {//instructions that use memory
				if self.pipe_mem_user == PipeMemUser::Free || matches!(self.pipe_mem_user, PipeMemUser::Execute(id) if id == self.execution_units[exe_index].id) {
					match self.execution_units[exe_index].ir.0 {
						Opcode::LDAa => {
							if let Ok(Some(num)) = self.read(u16::from_le_bytes([self.execution_units[exe_index].ir.1, self.execution_units[exe_index].ir.2]), PipeMemUser::Execute(self.execution_units[exe_index].id)) {
								self.a = num;
								self.set_zero(self.a);
								self.set_negative(self.a);
								self.execution_units[exe_index].busy = false;
								self.pipe_mem_user = PipeMemUser::Complete;
								self.instruction_counter += 1;
							}
						}
						Opcode::STAa => {
							self.write(u16::from_le_bytes([self.execution_units[exe_index].ir.1, self.execution_units[exe_index].ir.2]), self.a, PipeMemUser::Execute(self.execution_units[exe_index].id));
							self.execution_units[exe_index].busy = false;
							self.pipe_mem_user = PipeMemUser::Complete;
							self.instruction_counter += 1;
						}
						Opcode::ADCa => {
							if let Ok(Some(num)) = self.read(u16::from_le_bytes([self.execution_units[exe_index].ir.1, self.execution_units[exe_index].ir.2]), PipeMemUser::Execute(self.execution_units[exe_index].id)) {
								let (result, overflow) = self.a.overflowing_add(num);
								self.set_zero(result);
								self.set_negative(result);
								self.set_carry(result <= self.a && num != 0);
								if overflow {
									self.nv_bdizc |= Self::OVERFLOW_FLAG;
								} else {
									self.nv_bdizc &= !Self::OVERFLOW_FLAG;
								}
								self.a = result;
								self.execution_units[exe_index].busy = false;
								self.pipe_mem_user = PipeMemUser::Complete;
								self.instruction_counter += 1;
							}
						}
						Opcode::LDXa => {
							if let Ok(Some(num)) = self.read(u16::from_le_bytes([self.execution_units[exe_index].ir.1, self.execution_units[exe_index].ir.2]), PipeMemUser::Execute(self.execution_units[exe_index].id)) {
								self.x = num;
								self.set_zero(self.x);
								self.set_negative(self.x);
								self.execution_units[exe_index].busy = false;
								self.pipe_mem_user = PipeMemUser::Complete;
								self.instruction_counter += 1;
							}
						}
						Opcode::LDYa => {
							if let Ok(Some(num)) = self.read(u16::from_le_bytes([self.execution_units[exe_index].ir.1, self.execution_units[exe_index].ir.2]), PipeMemUser::Execute(self.execution_units[exe_index].id)) {
								self.y = num;
								self.set_zero(self.y);
								self.set_negative(self.y);
								self.execution_units[exe_index].busy = false;
								self.pipe_mem_user = PipeMemUser::Complete;
								self.instruction_counter += 1;
							}
						}
						Opcode::CPXa => {
							if let Ok(Some(num)) = self.read(u16::from_le_bytes([self.execution_units[exe_index].ir.1, self.execution_units[exe_index].ir.2]), PipeMemUser::Execute(self.execution_units[exe_index].id)) {
								let difference: u8 = self.x.wrapping_sub(num);
								self.set_zero(difference);
								self.set_negative(difference);
								self.set_carry(self.x >= num);
								self.execution_units[exe_index].busy = false;
								self.pipe_mem_user = PipeMemUser::Complete;
								self.instruction_counter += 1;
							}
						}
						Opcode::INCa => {
							let addr: u16 = u16::from_le_bytes([self.execution_units[exe_index].ir.1, self.execution_units[exe_index].ir.2]);
							if let Ok(Some(mut num)) = self.read(addr, PipeMemUser::Execute(self.execution_units[exe_index].id)) {
								num = num.wrapping_add(1);
								self.set_zero(num);
								self.set_negative(num);
								self.write(addr, num, PipeMemUser::Execute(self.execution_units[exe_index].id));
								self.execution_units[exe_index].busy = false;
								self.pipe_mem_user = PipeMemUser::Complete;
								self.instruction_counter += 1;
							}
						}
						Opcode::SYS if self.x != 1 => {
							match self.x {
								0x02 => {
									let addr: u16 = u16::from_le_bytes([self.execution_units[exe_index].ir.1, self.execution_units[exe_index].ir.2]).wrapping_add(self.y as u16);
									if let Ok(Some(num)) = self.read(addr, PipeMemUser::Execute(self.execution_units[exe_index].id)) {
										Self::sys_out_char(ascii::ENCODER.get(&num).unwrap_or(&'\0').clone());
										self.execution_units[exe_index].busy = false;
										self.pipe_mem_user = PipeMemUser::Complete;
										self.instruction_counter += 1;
									}
								}
								0x03 => {
									if let Ok(Some(num)) = self.read(u16::from_le_bytes([self.execution_units[exe_index].ir.1, self.execution_units[exe_index].ir.2]), PipeMemUser::Execute(self.execution_units[exe_index].id)) {
										let c: char = ascii::ENCODER.get(&num).unwrap_or(&'\0').clone();
										if c != '\0' {
											Self::sys_out_char(c);
											let (result, overflow) = self.execution_units[exe_index].ir.1.overflowing_add(1);
											self.execution_units[exe_index].ir.1 = result;
											if overflow {
												self.execution_units[exe_index].ir.2 = self.execution_units[exe_index].ir.2.wrapping_add(1);
											}
										} else {
											self.execution_units[exe_index].busy = false;
											self.pipe_mem_user = PipeMemUser::Complete;
											self.instruction_counter += 1;
										}
									}
								}
								_ => {panic!("Invalid arguments for system call")}
							}
						}
						_ => {}
					}
				}
			}
		}
	}
	///Immediately prints the ASCII representation of the byte in the device's output buffer
	fn interrupt_check(&mut self) {
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
						Self::sys_out_char(c.to_owned());
					}
				} else {
					panic!("Could not find I/O device Name: {}, IQR: {}", event.name, event.iqr);
				}
			}
		}
	}
}

struct ExecutionUnit {
	///Index in the cpu's array
	id: u8,
	///Points to the byte after the last byte of the instruction
	ip: u16,
	ir: (Opcode, u8, u8),
	busy: bool
}

impl ExecutionUnit {
	fn new(id: u8) -> Self {
		Self {
			id,
			ip: 0x00,
			ir: (Opcode::BRK, 0x00, 0x00),
			busy: false
		}
	}
	fn set_instruction(&mut self, ip: u16, ir: (Opcode, u8, u8)) {
		self.ip = ip;
		self.ir = ir;
		self.busy = true;
	}
}

/**6502 ASM opcodes.
Lowercase 4th letter indicates addressing mode.
i = immediate, a = absolute, r = relative, none = accumulator.
Parameters indicate operands.*/
#[repr(u8)]
#[derive(Debug, PartialEq, Clone)]
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
		match opcode {
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
	///All the storages that this instruction MAY affect
	fn affected_storage(&self) -> Vec<Storage> {
		match self {
			Opcode::LDAi => {vec![Storage::A, Storage::ZeroFlag]}
			Opcode::LDAa => {vec![Storage::A, Storage::ZeroFlag]}
			Opcode::STAa => {vec![Storage::Memory]}
			Opcode::TXA => {vec![Storage::A, Storage::ZeroFlag]}
			Opcode::TYA => {vec![Storage::A, Storage::ZeroFlag]}
			Opcode::ADCa => {vec![Storage::A, Storage::ZeroFlag]}
			Opcode::LDXi => {vec![Storage::X, Storage::ZeroFlag]}
			Opcode::LDXa => {vec![Storage::X, Storage::ZeroFlag]}
			Opcode::TAX => {vec![Storage::X, Storage::ZeroFlag]}
			Opcode::LDYi => {vec![Storage::Y, Storage::ZeroFlag]}
			Opcode::LDYa => {vec![Storage::Y, Storage::ZeroFlag]}
			Opcode::TAY => {vec![Storage::Y, Storage::ZeroFlag]}
			Opcode::CPXa => {vec![Storage::ZeroFlag]}
			Opcode::BNEr => {vec![Storage::PC]}
			Opcode::INCa => {vec![Storage::Memory, Storage::ZeroFlag]}
			_ => {vec![]}
		}
	}
	///All the storages that this instruction MAY depend on
	fn dependent_storage(&self) -> Vec<Storage> {
		match self {
			Opcode::LDAa => {vec![Storage::Memory]}
			Opcode::STAa => {vec![Storage::A]}
			Opcode::TXA => {vec![Storage::X]}
			Opcode::TYA => {vec![Storage::Y]}
			Opcode::ADCa => {vec![Storage::A, Storage::Memory]}
			Opcode::LDXa => {vec![Storage::Memory]}
			Opcode::TAX => {vec![Storage::A]}
			Opcode::LDYa => {vec![Storage::Memory]}
			Opcode::TAY => {vec![Storage::A]}
			Opcode::CPXa => {vec![Storage::X, Storage::Memory]}
			Opcode::BNEr => {vec![Storage::ZeroFlag]}
			Opcode::INCa => {vec![Storage::Memory]}
			Opcode::SYS => {vec![Storage::X, Storage::Y, Storage::Memory]}
			_ => {vec![]}
		}
	}
}

///Tracks which part of the CPU is using the memory between clock cycles to prevent data races and data loss
#[derive(PartialEq, Clone)]
enum PipeMemUser {
	Fetch,
	Decode,
	Execute(u8),
	///Users set pipe_mem_user to Complete when they finish, as opposed to Free because only one user can access memory in a cycle
	Complete,
	///If pipe_mem_user is Complete at the end of a cpu cycle, it is set to free to allow users to access memory in the next cycle
	Free
}

#[derive(PartialEq)]
enum Storage {
	A,
	X,
	Y,
	ZeroFlag,
	Memory,
	PC
}