use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener
};

/**Contains 0x10000 memory addresses in RAM.*/
pub struct Memory {
	pub specs: HardwareSpecs,
	ram: Box<[u8; 0x10000]>//unique_ptr because it's too big for the stack
}

impl Hardware for Memory {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let memory: Self = Self {
			specs: HardwareSpecs::new_default("Memory"),
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
	}
}

impl Memory {
	fn reset(&mut self) {
		self.ram = Box::new([0x00; 0x10000]);
	}
	
	//TODO change read and write so it works on its own cycle in parallel with the CPU
	/**Reads the value at the address of the MAR and stores it into the MDR.*/
	pub fn read(&mut self, mar: u16, mdr: &mut u8) {*mdr = self.ram[mar as usize];}
	/**Writes the value in the MDR into the address the MAR.*/
	pub fn write(&mut self, mar: u16, mdr: u8) {self.ram[mar as usize] = mdr;}
	
	
	//TODO refactor these functions for Lab 03 Milestone II
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