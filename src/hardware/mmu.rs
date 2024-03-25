use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	memory::Memory
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
	//TODO in the CPU pulse, I need to call these instead of Memory::read/write
	pub fn read() {}
	pub fn write() {}
}