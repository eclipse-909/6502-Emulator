use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener
};

pub struct Memory {
	specs: HardwareSpecs,
	ram: [u8; 0x10000]
}

impl Hardware for Memory {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let memory: Self = Self {
			specs: HardwareSpecs::new_default("Memory"),
			ram: [0x00; 0x10000],
		};
		memory.log("Created");
		return memory;
	}
}

impl ClockListener for Memory {
	fn pulse(&mut self) {self.log("Received clock pulse");}
}

impl Memory {
	/**Displays the hex values at each memory address in the range first..=last.*/
	pub fn display_memory(&self, first: u16, last: u16) {
		for i in first..=last {
			self.log(format!("Address 0x{:04x} holds value 0x{:02X}", i, self.ram[i as usize]).as_str());
		}
	}
}