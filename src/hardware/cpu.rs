use crate::hardware::{
	hardware::{Hardware, HardwareSpecs},
	imp::clock_listener::ClockListener
};

pub struct Cpu {
	specs: HardwareSpecs,
	cpu_clock_counter: u128,
	PC: u16,
	SP: u16,
	A: u8,
	X: u8,
	Y: u8,
	NV_BDIZC: u8
}

impl Hardware for Cpu {
	fn get_specs(&self) -> &HardwareSpecs {return &self.specs;}
	
	fn new() -> Self {
		let cpu: Self = Self {
			specs: HardwareSpecs::new_default("Cpu"),
			cpu_clock_counter: 0,
			PC: 0xfffc,//0xfffc and 0xfffd hold the address that the program starts at
			SP: 0x01ff,//stack grows down
			A: 0x00,
			X: 0x00,
			Y: 0x00,
			NV_BDIZC: 0b00110001
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