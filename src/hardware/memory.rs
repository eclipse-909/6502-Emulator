use {
	crate::hardware::{
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener,
		cache::INDEX_MASK
	},
	tokio::sync::mpsc::{Sender, Receiver, error::TryRecvError}
};

pub const N_WAYS: u8 = INDEX_MASK + 1;

/**Contains 0x10000 memory addresses in RAM. Interleaved n-ways.
MAR and MDR are not variables, but are represented in the channel*/
pub struct Memory {
	pub specs: HardwareSpecs,
	/**Represents bus lines from Memory to Cache.*/
	tx: Sender<MemEvent>,
	/**Represents bus lines from Cache to Memory.*/
	rx: Receiver<MemEvent>,
	ram: Box<[u8; 0x10000 / N_WAYS as usize]>//unique_ptr because it's too big for the stack
}

impl Hardware for Memory {
	fn get_specs(&self) -> &HardwareSpecs {&self.specs}
}

impl ClockListener for Memory {
	fn pulse(&mut self) {
		self.log("Received clock pulse");
		//see if the MMU requested a read or write
		match self.rx.try_recv() {
			Ok(MemEvent::MemReadRequest{mar}) => {
				self.tx.try_send(MemEvent::MemReadResponse{mdr: self.ram[mar as usize]}).expect("Cache receiver buffer full");
			}
			Ok(MemEvent::MemWriteRequest{mar, mdr}) => {
				self.ram[mar as usize] = mdr;
				self.tx.try_send(MemEvent::MemWriteResponse).expect("Cache receiver buffer full");//Cache is supposed to clear the buffer before sending requests to memory
			}
			Err(TryRecvError::Empty) => {}//no memory action needed this cycle
			_ => {
				panic!("Received invalid value from memory receiver");
			}
		}
	}
}

impl Memory {
	pub fn new(tx: Sender<MemEvent>, rx: Receiver<MemEvent>) -> Self {
		let memory: Self = Self {
			specs: HardwareSpecs::new("Memory"),
			tx,
			rx,
			ram: Box::new([0x00; 0x10000 / N_WAYS as usize]),
		};
		memory.log(format!("Created - Addressable Range: 0x{:04X}", 0x10000 / N_WAYS as usize).as_str());
		memory
	}
	
	/**Fills RAM (save the reset vector) with 0x00.*/
	pub fn reset(&mut self) {
		for i in 0x0000..0xFFFC {
			self.ram[i] = 0x00;
		}
	}
}

#[derive(Debug)]
pub enum MemEvent {
	MemReadRequest{mar: u16},
	MemWriteRequest{mar: u16, mdr: u8},
	MemReadResponse{mdr: u8},
	MemWriteResponse
}