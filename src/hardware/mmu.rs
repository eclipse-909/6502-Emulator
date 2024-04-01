use {
	crate::hardware::{
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener,
		memory::Memory
	},
	std::sync::mpsc::{Receiver, Sender}
};

/*TODO figure out if the MMU struct is necessary for the lab
I can put all this stuff directly in the CPU struct and it would be a little easier to maintain and less verbose
*/
pub struct Mmu {
	specs: HardwareSpecs,
	/**Represents bus lines from MMU to Memory. (mar, mdr, read = false / write = true)*/
	tx: Sender<(u16, u8, bool)>,
	/**Represents bus lines from Memory to MMU. The u8 represents the value the MDR should hold.*/
	pub rx: Receiver<u8>
}

impl Hardware for Mmu {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	fn get_specs_mut(&mut self) -> &mut HardwareSpecs {return &mut self.specs;}
}

impl Mmu {
	pub fn new(tx: Sender<(u16, u8, bool)>, rx: Receiver<u8>) -> Self {
		let mmu: Self = Self {
			specs: HardwareSpecs::new("MMU"),
			tx,
			rx
		};
		mmu.log("Created");
		return mmu;
	}
	/**Requests the Memory to perform a read operation on its next cycle. To access the value that was read, use rx.recv() or rx.try_recv().
	 Should not be called on back-to-back cycles with itself or Self::write().*/
	pub fn read(&self, mar: u16) {self.tx.send((mar, 0, false)).expect("Error sending MAR to memory unit.");}
	/**Requests the Memory to perform a write operation on its next cycle. Should not be called on back-to-back cycles with itself or Self::read()*/
	pub fn write(&self, mar: u16, mdr: u8) {self.tx.send((mar, mdr, true)).expect("Error sending MAR and MDR to memory unit.");}
	
	//TODO figure out if these functions are sufficient for the lab requirements
	/**Takes a u8 slice and flashes it into RAM. The first value in the slice is stored at start_addr.*/
	pub fn static_load(&self, memory: &mut Memory, values: &[u8], start_addr: u16) {
		for (i, val) in values.iter().enumerate() {
			self.write(start_addr + i as u16, *val);
			memory.pulse();
		}
	}
	/**Logs the values at each memory address in the range start_addr..end_addr*/
	pub fn memory_dump(&self, memory: &mut Memory, start_addr: u16, end_addr: u16) {
		for i in start_addr..end_addr {
			self.read(i);
			memory.pulse();
			match self.rx.try_recv() {
				Ok(mdr) => {
					self.log(format!("Address 0x{:04x}: | 0x{:02X}", i, mdr).as_str());
				}
				Err(e) => {
					self.log(format!("Error dumping memory. Attempted to access memory at address 0x{:04x}. Error message: {e}", i).as_str());
					return;
				}
			}
		}
	}
}