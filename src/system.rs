use {
	std::time::Duration,
	tokio::time::sleep,
	crate::{
		hardware::{
			clock::Clock,
			cpu::Cpu,
			hardware::{Hardware, HardwareSpecs},
			imp::clock_listener::ClockListener,
		},
		lib
	}
};

pub struct System {
	specs: HardwareSpecs,
	pub clock: Clock
}

impl Hardware for System {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	fn get_specs_mut(&mut self) -> &mut HardwareSpecs {return &mut self.specs;}
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
		return system;
	}
	
	/**Loads a set of instructions into memory and tells the cpu to start there. Must be called before System::start()*/
	pub fn load_main_program(&mut self, address: u16, program: &[u8]) {
		self.clock.cpu.mmu.static_load(&mut self.clock.memory, program, address);
		let le_address: (u8,u8) = lib::u16_to_little_endian(address);
		self.clock.cpu.mmu.static_load(&mut self.clock.memory, &[le_address.0, le_address.1], 0xFFFC);//set the reset vector
	}
	
	/**Starts the system and begins processing instructions until BRK.*/
	pub async fn start(&mut self) {
		self.clock.cpu.mmu.memory_dump(&mut self.clock.memory, 0x0000, 0x0015);
		self.log("The delay between cycles has been greatly reduced to speed up the program.");
		self.read_reset_vector();
		
		self.log("Program Output:\n===================================================================================");
		while self.clock.cpu.nv_bdizc & Cpu::BREAK_FLAG != Cpu::BREAK_FLAG {
			self.clock.pulse();
			
			/* ATTENTION!!!!!
			If sleep is commented out, the program will run almost instantly.
			If you include sleep, it will take a little over 5 minutes to run the whole program, even if the interval is 0.
			If the interval is more than 0, I can't imagine how long it would take.
			*/
			
			//I'm going to make it sleep for 0 second every 100 pulses, so it will only take a few seconds to run.
			if self.clock.cpu.cpu_clock_counter % 100 == 0 {
				sleep(Duration::from_micros(Self::CLOCK_INTERVAL_MICRO)).await; //uncomment to include the delay between cycles
			}
		}
		println!("\n===================================================================================");
		self.clock.cpu.specs.debug = true;
		self.clock.cpu.log(format!("Total CPU clock cycles: {}", self.clock.cpu.cpu_clock_counter).as_str());
	}
	
	/**Resets the RAM and sets the program counter back to the reset vector.*/
	fn restart(&mut self) {
		self.clock.memory.reset();
		self.read_reset_vector();
	}
	/**Reads the address of the reset vector and loads it into the program counter*/
	fn read_reset_vector(&mut self) {
		self.clock.cpu.mar = 0xFFFC;
		self.clock.cpu.mmu.read(self.clock.cpu.mar);
		self.clock.memory.pulse();
		self.clock.cpu.check_read_ready();
		self.clock.cpu.pc = (self.clock.cpu.pc & 0xFF00) | (self.clock.cpu.mdr as u16);
		self.clock.cpu.mar = 0xFFFD;
		self.clock.cpu.mmu.read(self.clock.cpu.mar);
		self.clock.memory.pulse();
		self.clock.cpu.check_read_ready();
		self.clock.cpu.pc = (self.clock.cpu.pc & 0x00FF) | ((self.clock.cpu.mdr as u16) << 8);
	}
}