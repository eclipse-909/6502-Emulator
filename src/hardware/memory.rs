use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener
};

/**Contains 0x10000 memory addresses in RAM.*/
pub struct Memory {
	pub specs: HardwareSpecs,
	mar: u16,
	mdr: u8,
	ram: Box<[u8; 0x10000]>//unique_ptr because it's too big for the stack
}

impl Hardware for Memory {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let memory: Self = Self {
			specs: HardwareSpecs::new_default("Memory"),
			mar: 0x0000,
			mdr: 0x00,
			ram: Box::new([0x00; 0x10000])
		};
		memory.log("Created - Addressable space: 0 - 65535");
		return memory;
	}
}

impl ClockListener for Memory {
	//I'm not really sure what purpose this will serve, but I'll refactor my get, set, and set_range functions if I need to
	fn pulse(&mut self) {self.log("Received clock pulse");}
}

impl Memory {
	fn reset(&mut self) {
		self.mar = 0x0000;
		self.mdr = 0x00;
		self.ram = Box::new([0x00; 0x10000]);
	}
	
	fn read(&mut self) {self.mdr = self.ram[self.mar];}
	
	fn write(&mut self) {self.ram[self.mar] = self.mdr;}
	
	/**Displays the hex values at each memory address in the range first..=last.*/
	pub fn display_memory(&self, first: u16, last: u16) {
		for i in first..=last {
			self.log(format!("Address 0x{:04x} holds value 0x{:02X}", i, self.get(i)).as_str());
		}
	}
	
	
	//The below functions are old and will be replaced by functions above
	
	/**Gets the byte at the given address.*/
	pub fn get(&self, address: u16) -> u8 {
		return self.ram[address as usize];
	}
	
	/**Gets the bytes at the given address as an array slice with the given length*/
	pub fn get_range(&self, address: u16, len: u16) -> &[u8] {
		return &self.ram[address as usize .. address as usize + len as usize];
	}
	
	/**Sets the byte at the given address with the given value.*/
	pub fn set(&mut self, address: u16, value: u8) {
		self.ram[address as usize] = value;
	}
	
	/**Sets the bytes in memory to the corresponding values given.*/
	pub fn set_range(&mut self, address: u16, values: &[u8]) {
		for (i, value) in values.iter().enumerate() {
			self.ram[address as usize + i] = *value;
		}
	}
}