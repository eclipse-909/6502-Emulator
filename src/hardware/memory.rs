use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener
};

/**Contains RAM, I/O, and ROM. Use get and set functions to use memory. Accessing non-addressable memory may lead to undefined behavior.*/
pub struct Memory {
	specs: HardwareSpecs,
	
	/**
	Addressable: 0x0000 - 0x7fff
	
	Stack: 0x0100 - 0x01ff
	 */
	ram: [u8; 0x8000],
	
	/**
	Addressable:
	 * 0x8010 - 0x801f
	 * 0x8020 - 0x802f
	 * 0x8040 - 0x804f
	 * 0x8080 - 0x808f
	 * 0x8100 - 0x810f
	 * 0x8200 - 0x820f
	 * 0x8400 - 0x840f
	 * 0x8800 - 0x880f
	 * 0x9000 - 0x900f
	 */
	io: [u8; 0x1000],
	
	/**Addressable: 0xa000 - 0xffff*/
	rom: [u8; 0x6000]
}

impl Hardware for Memory {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let memory: Self = Self {
			specs: HardwareSpecs::new_default("Memory"),
			ram: [0x00; 0x8000],
			io: [0x00; 0x1000],
			rom: [0x00; 0x6000]
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
			self.log(format!("Address 0x{:04x} holds value 0x{:02X}", i, self.get(i)).as_str());
		}
	}
	
	/**Gets the byte at the given address.*/
	pub fn get(&self, address: u16) -> u8 {
		let address: usize = address as usize;
		return if address < 0x8000 {
			self.ram[address]
		} else if address < 0x9010 {
			self.io[address - 0x8010]
		} else {
			self.rom[address - 0xa000]
		}
	}
	
	/**Sets the byte at the given address with the given value.*/
	pub fn set(&mut self, address: u16, value: u8) {
		let address: usize = address as usize;
		if address < 0x8000 {
			self.ram[address] = value;
		} else if address < 0x9010 {
			self.io[address - 0x8010] = value;
		} else {
			self.rom[address - 0xa000] = value;
		}
	}
}