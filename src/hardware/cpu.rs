use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener
};

pub struct Cpu {
	specs: HardwareSpecs,
	cpu_clock_counter: u128
}

impl Hardware for Cpu {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let cpu: Self = Self {
			specs: HardwareSpecs::new_default("Cpu"),
			cpu_clock_counter: 0
		};
		cpu.log("Created");
		return cpu;
	}
}

impl ClockListener for Cpu {
	fn pulse(&mut self) {
		self.log(format!("Received clock pulse - CPU clock count: {}", self.cpu_clock_counter).as_str());
		self.cpu_clock_counter += 1;
	}
}