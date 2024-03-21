use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	memory::{Memory, MemState}
};

pub struct MMU {
	specs: HardwareSpecs,
	pub memory: Memory
}

impl Hardware for MMU {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let mmu: Self = Self {
			specs: HardwareSpecs::new_default("MMU"),
			memory: Memory::new()
		};
		mmu.log("Created");
		return mmu;
	}
}

impl MMU {
	/**Sets the low byte of the MAR.*/
	pub fn set_mar_low(&mut self, low_byte: u8) {self.memory.mar = (self.memory.mar & 0xFF00) | (low_byte as u16);}
	/**Sets the high byte of the MAR.*/
	pub fn set_mar_high(&mut self, high_byte: u8) {self.memory.mar = (self.memory.mar & 0x00FF) | ((high_byte as u16) << 8);}
	
	/*TODO change how the memory decides which cycle to perform a the operation on.
	I should pass in the pipeline_step and in Memory::read/write() I should match the pipeline to only read/write on certain steps
	*/
	/**Tells the memory to read during the NEXT CYCLE.*/
	pub fn read(&mut self) {self.memory.state = MemState::WaitRead;}
	/**Tells the memory to write during the NEXT CYCLE.*/
	pub fn write(&mut self) {self.memory.state = MemState::WaitWrite;}
}