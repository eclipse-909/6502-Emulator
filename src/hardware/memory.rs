use {
	crate::hardware::{
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener
	},
	tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver}
};

/**Contains 0x10000 memory addresses in RAM.*/
pub struct Memory {
	specs: HardwareSpecs,
	/**Represents bus lines from Memory to MMU. The u8 represents the value the MDR should hold.*/
	tx: UnboundedSender<u8>,
	/**Represents bus lines from MMU to Memory. (mar, mdr, read = false / write = true)*/
	rx: UnboundedReceiver<(u16, u8, bool)>,
	ram: Box<[u8; 0x10000]>,//unique_ptr because it's too big for the stack
	//TODO fix the memory so that it is NOT memory mapped. IO will be handled by passing the data through the interrupt controller.
}

impl Hardware for Memory {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	fn get_specs_mut(&mut self) -> &mut HardwareSpecs {return &mut self.specs;}
}

impl ClockListener for Memory {
	fn pulse(&mut self) {
		self.log("Received clock pulse");
		//see if the MMU requested a read or write
		match self.rx.try_recv() {
			Ok((mar, _, false)) => {//false = read
				self.tx.send(self.ram[mar as usize]).expect("Error sending memory data to MDR.");
			}
			Ok((mar, mdr, true)) => {//true = write
				self.ram[mar as usize] = mdr;
			}
			_ => {}//no memory action needed this cycle
		}
	}
}

impl Memory {
	pub fn new(tx: UnboundedSender<u8>, rx: UnboundedReceiver<(u16, u8, bool)>) -> Self {
		let memory: Self = Self {
			specs: HardwareSpecs::new("Memory"),
			tx,
			rx,
			ram: Box::new([0x00; 0x10000]),
		};
		memory.log("Created - RAM: 0x0000-0xFFFF");
		return memory;
	}
	
	/**Fills RAM (save the reset vector) with 0x00.*/
	pub fn reset(&mut self) {
		for i in 0x0000..0xFFFC {
			self.ram[i] = 0x00;
		}
	}
}