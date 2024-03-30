use {
	crate::hardware::{
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener
	},
	std::sync::mpsc::{Receiver, Sender}
};

/**Contains 0x10000 memory addresses in RAM.*/
pub struct Memory {
	pub specs: HardwareSpecs,
	//represents bus lines
	tx: Sender<u8>,//mdr
	rx: Receiver<(u16, u8, Action)>,//(mar, mdr, action)
	ram: Box<[u8; 0x10000]>//unique_ptr because it's too big for the stack
}

#[repr(u8)]
pub enum Action {None, Read, Write}

impl Hardware for Memory {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
}

impl ClockListener for Memory {
	fn pulse(&mut self) {
		self.log("Received clock pulse");
		//see if the cpu requested a read or write
		match self.rx.try_recv() {
			Ok((mar, _, Action::Read)) => {
				self.tx.send(self.ram[mar as usize]).expect("Error sending memory data to MDR.");
			}
			Ok((mar, mdr, Action::Write)) => {
				self.ram[mar as usize] = mdr;
			}
			_ => {}
		}
	}
}

impl Memory {
	pub fn new(tx: Sender<u8>, rx: Receiver<(u16, u8, Action)>) -> Self {
		let memory: Self = Self {
			specs: HardwareSpecs::new("Memory"),
			tx,
			rx,
			ram: Box::new([0x00; 0x10000])
		};
		memory.log("Created - Addressable space: 0 - 65535");
		return memory;
	}
	
	fn reset(&mut self) {
		self.ram = Box::new([0x00; 0x10000]);
	}
	
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