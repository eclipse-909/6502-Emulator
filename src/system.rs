use {
	tokio::time::{sleep, Duration},
	crate::hardware::{
		clock::Clock,
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener
	}
};

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
	const CLOCK_INTERVAL_MICRO: u64 = 10_000;//number is in microseconds or 0.001 milliseconds
	
	/**Loads a set of instructions into memory and tells the cpu to start there.*/
	pub fn load_main_program(&mut self, address: u16, program: &[u8]) {
		let le_address: (u8,u8) = Self::u16_to_little_endian(&address);
		self.clock.cpu.memory.set_range(0xfffc, &[le_address.0, le_address.1]);
		self.clock.cpu.memory.set_range(address, program);
	}
	
	/**loads a set of instructions into memory.*/
	pub fn load_program(&mut self, address: u16, program: &[u8]) {self.clock.cpu.memory.set_range(address, program);}
	
	pub async fn start_system(&mut self) {
		self.clock.cpu.running = true;
		self.clock.cpu.memory.display_memory(0x0000, 0x0014);
		let (i, ii) = (self.clock.cpu.fetch(), self.clock.cpu.fetch());
		self.clock.cpu.PC = Self::little_endian_to_u16(i, ii);
		self.clock.cpu.specs.debug = false;
		self.clock.cpu.memory.specs.debug = false;
		loop {
			if !self.clock.cpu.running {return;}
			self.clock.pulse();
			sleep(Duration::from_micros(Self::CLOCK_INTERVAL_MICRO)).await;
		}
	}
}