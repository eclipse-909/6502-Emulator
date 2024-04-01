use crate::{
	lib,
	hardware::{
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
	fn get_specs_mut(&mut self) -> &mut HardwareSpecs {return &mut self.specs;}
}

impl System {
	/**number is in microseconds or 0.001 milliseconds*/
	const CLOCK_INTERVAL_MICRO: u64 = 1_000;//the thread.sleep command has been commented out, so there is no delay between cycles
	
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
	pub fn start(&mut self) {
		self.clock.cpu.mmu.memory_dump(&mut self.clock.memory, 0x0000, 0x0015);
		self.log("The delay between cycles has been disabled to speed up the program.");
		self.read_reset_vector();
		
		self.log("\n\nProgram Output:\n===================================================================================");
		while self.clock.cpu.nv_bdizc & Cpu::BREAK_AND_INTERRUPT_FLAG != Cpu::BREAK_AND_INTERRUPT_FLAG {
			self.clock.pulse();
			//sleep(Duration::from_micros(Self::CLOCK_INTERVAL_MICRO)); //uncomment to include the delay between cycles
		}
		println!("\n");
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