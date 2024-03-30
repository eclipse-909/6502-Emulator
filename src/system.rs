use crate::hardware::{
	clock::Clock,
	cpu::Cpu,
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener
};

pub struct System {
	specs: HardwareSpecs,
	clock: Clock
}

impl Hardware for System {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
}

impl System {
	//the thread.sleep command has been commented out, so there is no delay between cycles
	const CLOCK_INTERVAL_MICRO: u64 = 1_000;//number is in microseconds or 0.001 milliseconds
	
	pub fn new() -> Self {
		let system: Self = Self {
			specs: HardwareSpecs::new("System"),
			clock: Clock::new()
		};
		system.log("Created");
		return system;
	}
	
	/**Loads a set of instructions into memory and tells the cpu to start there.*/
	pub fn load_main_program(&mut self, address: u16, program: &[u8]) {
		let le_address: (u8,u8) = Self::u16_to_little_endian(&address);
		self.clock.memory.set_range(0xfffc, &[le_address.0, le_address.1]);
		self.clock.memory.set_range(address, program);
	}
	
	pub fn start_system(&mut self) {
		self.clock.memory.display_memory(0x0000, 0x0014);
		self.log("The delay between cycles has been disabled to speed up the program.");
		self.clock.cpu.pc = 0x00;//doing it this way for now
		self.clock.cpu.specs.debug = false;
		self.clock.memory.specs.debug = false;
		while self.clock.cpu.nv_bdizc & Cpu::BREAK_AND_INTERRUPT_FLAG != Cpu::BREAK_AND_INTERRUPT_FLAG {
			self.clock.pulse();
			//sleep(Duration::from_micros(Self::CLOCK_INTERVAL_MICRO));
		}
		println!();
		self.clock.cpu.specs.debug = true;
		self.clock.cpu.log(format!("Total CPU clock cycles: {}", self.clock.cpu.cpu_clock_counter).as_str());
	}
}