use {
	std::time::Duration,
	tokio::time::sleep,
	crate::hardware::{
		clock::Clock,
		cpu::Cpu,
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener
	}
};

pub struct System {
	specs: HardwareSpecs,
	pub clock: Clock
}

impl Hardware for System {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
}

impl System {
	/**Delay between clock pulses (in microseconds or 0.001 milliseconds)*/
	const CLOCK_INTERVAL_MICRO: u64 = 0;//the thread.sleep command has been commented out, so there is no delay between cycles
	
	/**Instantiates a new System object.*/
	pub fn new() -> Self {
		let system: Self = Self {
			specs: HardwareSpecs::new("System"),
			clock: Clock::new()
		};
		system.log("Created");
		system
	}
	
	/**Loads a set of instructions into memory and tells the cpu to start there. Must be called before System::start()*/
	pub fn load_main_program(&mut self, address: u16, program: &[u8]) {
		self.clock.cpu.mmu.static_load(&mut self.clock.memory, program, address);
	}
	
	/**Starts the system and begins processing instructions until BRK.*/
	pub async fn start(&mut self) {
		self.clock.cpu.mmu.memory_dump(&mut self.clock.memory, 0x0000, 0x0015);
		self.clock.cpu.mmu.cache.cache_hits = 0;
		self.clock.cpu.mmu.cache.cache_accesses = 0;
		self.log("The delay between cycles has been greatly reduced to speed up the program.");
		
		self.log("Program Output:\n===================================================================================");
		while self.clock.cpu.nv_bdizc & Cpu::BREAK_FLAG != Cpu::BREAK_FLAG {
			self.clock.pulse();
			
			/* ATTENTION!!!!!
			If sleep is commented out, the program will run almost instantly.
			If you include sleep, it will take a little over 5 minutes to run the whole program, even if the interval is 0.
			If the interval is more than 0, I can't imagine how long it would take.
			*/
			
			//I'm going to make it sleep for 0 second every 30 pulses, so it will only take a few seconds to run.
			if self.clock.cpu.cpu_clock_counter % 30 == 0 {
				sleep(Duration::from_micros(Self::CLOCK_INTERVAL_MICRO)).await; //uncomment to include the delay between cycles
			}
		}
		println!("\n===================================================================================");
		self.clock.cpu.specs.debug = true;
		self.clock.cpu.log(format!("Total CPU clock cycles: {}", self.clock.cpu.cpu_clock_counter).as_str());
		self.clock.cpu.log(format!("Total CPU instructions executed: {}", self.clock.cpu.instruction_counter).as_str());
		self.clock.cpu.log(format!("Instructions per clock cycle: {}", self.clock.cpu.instruction_counter as f32 / self.clock.cpu.cpu_clock_counter as f32).as_str());
		self.clock.cpu.mmu.cache.log(format!("Total cache hits: {}", self.clock.cpu.mmu.cache.cache_hits).as_str());
		self.clock.cpu.mmu.cache.log(format!("Total cache accesses: {}", self.clock.cpu.mmu.cache.cache_accesses).as_str());
		self.clock.cpu.mmu.cache.log(format!("Cache hit ratio: {}", self.clock.cpu.mmu.cache.cache_hits as f32 / self.clock.cpu.mmu.cache.cache_accesses as f32).as_str());
	}
	
	/**Resets the RAM and sets the program counter back to the reset vector.*/
	fn restart(&mut self) {
		self.clock.memory.iter_mut().for_each(|mem| {mem.reset();});
		self.clock.cpu.pc = 0x0000;
		self.clock.cpu.clear_pipeline();
	}
}