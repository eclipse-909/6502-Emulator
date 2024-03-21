use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener
};

/**Contains 0x10000 memory addresses in RAM.*/
pub struct Memory {
	pub specs: HardwareSpecs,
	pub mar: u16,
	pub mdr: u8,
	pub state: MemState,
	ram: Box<[u8; 0x10000]>//unique_ptr because it's too big for the stack
}

#[repr(u8)]
pub enum MemState {
	None = 0x00,
	WaitRead,
	ReadyRead,
	WaitWrite,
	ReadyWrite
}

impl Hardware for Memory {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let memory: Self = Self {
			specs: HardwareSpecs::new_default("Memory"),
			mar: 0x0000,
			mdr: 0x00,
			state: MemState::None,
			ram: Box::new([0x00; 0x10000])
		};
		memory.log("Created - Addressable space: 0 - 65535");
		return memory;
	}
}

impl ClockListener for Memory {
	//I'm not really sure what purpose this will serve, but I'll refactor my get, set, and set_range functions if I need to
	fn pulse(&mut self) {
		self.log("Received clock pulse");
		match self.state {
			MemState::None => {}
			MemState::WaitRead => {
				self.state = MemState::ReadyRead;
			}
			MemState::ReadyRead => {
				self.read();
				self.state = MemState::None;
			}
			MemState::WaitWrite => {
				self.state = MemState::ReadyWrite;
			}
			MemState::ReadyWrite => {
				self.write();
				self.state = MemState::None;
			}
		}
	}
}

impl Memory {
	fn reset(&mut self) {
		self.mar = 0x0000;
		self.mdr = 0x00;
		self.ram = Box::new([0x00; 0x10000]);
	}
	/**Reads the value at the address of the MAR and stores it into the MDR.*/
	fn read(&mut self) {self.mdr = self.ram[self.mar as usize];}
	/**Writes the value in the MDR into the address the MAR.*/
	fn write(&mut self) {self.ram[self.mar as usize] = self.mdr;}
	
	/**Displays the hex values at each memory address in the range first..=last.*/
	pub fn display_memory(&self, first: u16, last: u16) {
		for i in first..=last {
			self.log(format!("Address 0x{:04x} holds value 0x{:02X}", i, self.ram[i as usize]).as_str());
		}
	}
	
	/**Sets the bytes in memory to the corresponding values given.*/
	pub fn set_range(&mut self, address: u16, values: &[u8]) {
		for (i, value) in values.iter().enumerate() {
			self.ram[address as usize + i] = *value;
		}
	}
}