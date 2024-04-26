use {
	crate::hardware::{
		cpu::Cpu,
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener,
		memory::Memory
	},
	tokio::sync::mpsc::unbounded_channel
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
	/**Pulses all registered hardware components.*/
	fn pulse(&mut self) {
		//These should technically pulse in parallel
		//If I spawn 2 threads to run these, it should work
		//CPU and Memory use mpsc Sender and Receiver to communicate with each other thread-safely
		self.cpu.pulse();
		self.memory.pulse();
		
		/*//Something like this should also work, but idk how cpu intensive it is to spawn a thread a couple of times per cycle
		let cpu_thread_handle: JoinHandle<()> = tokio::spawn(|self| {self.cpu.pulse();});
		let memory_thread_handle: JoinHandle<()> = tokio::spawn(|self| {self.memory.pulse();});
		cpu_thread_handle.await;
		memory_thread_handle.await;
		*/
		
		//to register more hardware components, just make them components of Clock and call their pulse functions here
	}
}

impl Clock {
	/**Creates a new instance and outputs "Created" to the log. The registered listeners = None.*/
	pub fn new() -> Self {
		let (cpu_tx, mem_rx) = unbounded_channel::<(u16, u8, bool)>();
		let (mem_tx, cpu_rx) = unbounded_channel::<u8>();
		let clock: Self = Self {
			specs: HardwareSpecs::new("Clock"),
			cpu: Cpu::new(cpu_tx, cpu_rx),
			memory: Memory::new(mem_tx, mem_rx)
		};
		clock.log("Created");
		return clock;
	}
}