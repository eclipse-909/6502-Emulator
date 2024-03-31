use {
	crate::hardware::{
		cpu::Cpu,
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener,
		memory::Memory
	},
	std::sync::mpsc
};

pub struct Clock {
	specs: HardwareSpecs,
	pub cpu: Cpu,
	pub memory: Memory
}

impl Hardware for Clock {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	fn get_specs_mut(&mut self) -> &mut HardwareSpecs {return &mut self.specs;}
}

impl ClockListener for Clock {
	fn pulse(&mut self) {
		//These should technically pulse in parallel
		//If I spawn 2 threads to run these, it should work
		//CPU and Memory use mpsc Sender and Receiver to communicate with each other thread-safely
		self.cpu.pulse();
		self.memory.pulse();
		//to register more hardware components, just make them components of Clock and call their pulse functions here
	}
}

impl Clock {
	/**Creates a new instance and outputs "Created" to the log. The registered listeners = None.*/
	pub fn new() -> Self {
		let (cpu_tx, mem_rx) = mpsc::channel::<(u16, u8, bool)>();
		let (mem_tx, cpu_rx) = mpsc::channel::<u8>();
		let clock: Self = Self {
			specs: HardwareSpecs::new("Clock"),
			cpu: Cpu::new(cpu_tx, cpu_rx),
			memory: Memory::new(mem_tx, mem_rx)
		};
		clock.log("Created");
		return clock;
	}
}