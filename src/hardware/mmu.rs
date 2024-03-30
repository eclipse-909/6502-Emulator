use {
	crate::hardware::{
		hardware::{Hardware, HardwareSpecs},
		memory::Action
	},
	std::sync::mpsc::{Receiver, Sender}
};

pub struct MMU {
	specs: HardwareSpecs,
	//represents bus lines
	tx: Sender<(u16, u8, Action)>,//(mar, mdr, action)
	pub rx: Receiver<u8>//mdr
}

impl Hardware for MMU {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
}

impl MMU {
	pub fn new(tx: Sender<(u16, u8, Action)>, rx: Receiver<u8>) -> Self {
		let mmu: Self = Self {
			specs: HardwareSpecs::new("MMU"),
			tx,
			rx
		};
		mmu.log("Created");
		return mmu;
	}
	
	//TODO in the CPU pulse, I need to call these instead of Memory::read/write
	pub fn read(&mut self, mar: u16) {self.tx.send((mar, 0, Action::Read)).expect("Error sending MAR to memory unit.");}
	pub fn write(&mut self, mar: u16, mdr: u8) {self.tx.send((mar, mdr, Action::Write)).expect("Error sending MAR and MDR to memory unit.");}
}