use crate::hardware::{
	clock::Clock,
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener,
	cpu::Cpu
};

/**This program is heavily object-oriented. The program is build around the System,
which is the top node in the hierarchy of composed objects.
System has a Clock, which has a Cpu, which has a Memory. I very much dislike this structure because
unnecessary encapsulation is the least scalable and most un-refactorable way to go about making this program.
I will likely try to change the structure in the future to accommodate the instructions that will be given in lab 2.*/
pub struct System {
	specs: HardwareSpecs,
	clock: Clock
}

impl Hardware for System {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let system: Self = Self {
			specs: HardwareSpecs::new_default("System"),
			clock: Clock::new()
		};
		system.log("Created");
		return system;
	}
}

impl System {
	const CLOCK_INTERVAL_MICRO: u64 = 1_000;//number is in microseconds or 0.001 milliseconds
	
	/**Loads a set of instructions into memory and tells the cpu to start there.*/
	pub fn load_main_program(&mut self, address: u16, program: &[u8]) {
		let le_address: (u8,u8) = Self::u16_to_little_endian(&address);
		self.clock.cpu.mmu.memory.set_range(0xfffc, &[le_address.0, le_address.1]);
		self.clock.cpu.mmu.memory.set_range(address, program);
	}
	
	pub fn start_system(&mut self) {
		self.clock.cpu.mmu.memory.display_memory(0x0000, 0x0014);
		self.clock.cpu.pc = 0x00;//doing it this way for now
		self.clock.cpu.specs.debug = false;
		self.clock.cpu.mmu.memory.specs.debug = false;
		while self.clock.cpu.nv_bdizc & Cpu::BREAK_FLAG != Cpu::BREAK_FLAG {
			self.clock.pulse();
			//sleep(Duration::from_micros(Self::CLOCK_INTERVAL_MICRO));
		}
	}
}