use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener,
	cpu::Cpu,
	memory::Memory
};

pub struct Clock {
	specs: HardwareSpecs,
	pub cpu: Cpu,
	pub memory: Memory,
}

impl Hardware for Clock {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	/**Creates a new instance and outputs "Created" to the log. The registered listeners = None.*/
	fn new() -> Self {
		let clock: Self = Self {
			specs: HardwareSpecs::new_default("Clock"),
			cpu: Cpu::new(),
			memory: Memory::new()
		};
		clock.log("Created");
		return clock;
	}
}

impl Clock {
	/**Called on each clock cycle. Calls the pulse method on each listener registered.*/
	pub fn invoke(&mut self) {
		self.cpu.pulse();
		self.memory.pulse();
	}
}