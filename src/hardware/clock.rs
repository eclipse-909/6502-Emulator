use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener,
	cpu::Cpu
};

pub struct Clock {
	specs: HardwareSpecs,
	pub cpu: Cpu
}

impl Hardware for Clock {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	/**Creates a new instance and outputs "Created" to the log. The registered listeners = None.*/
	fn new() -> Self {
		let clock: Self = Self {
			specs: HardwareSpecs::new_default("Clock"),
			cpu: Cpu::new()
		};
		clock.log("Created");
		return clock;
	}
}

impl ClockListener for Clock {
	/**Called on each clock cycle. Calls the pulse method on each listener registered.*/
	fn pulse(&mut self) {
		self.cpu.pulse();
	}
}