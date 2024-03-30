use std::time::Instant;

pub struct HardwareSpecs {
	id: u8,
	name: String,
	pub debug: bool
}

static mut START_TIME: Option<Instant> = None;

impl HardwareSpecs {
	/**Creates a new instance of HardwareSpecs. Defaults id to 0 and debug to true.*/
	pub fn new(name: &str) -> Self {
		return Self {
			id: 0,
			name: String::from(name),
			debug: true
		};
	}
}

pub trait Hardware {
	fn get_specs(&self) -> &HardwareSpecs;
	
	/**Gets the elapsed ms since the program started.*/
	fn elapsed_ms() -> u128 {
		unsafe {
			if START_TIME.is_none() {START_TIME = Some(Instant::now());}
			return Instant::now().duration_since(START_TIME.unwrap()).as_millis();
		}
	}
	
	/**Logs a message to the console with a specific format with hardware specs. Use this instead of println!() when printing.*/
	fn log(&self, message: &str) {
		if self.get_specs().debug {
			println!("ID: {} - Name: {} - Time: {:?} - Message: {}", self.get_specs().id, self.get_specs().name, Self::elapsed_ms(), message);
		}
	}
	
	fn u16_to_little_endian(value: &u16) -> (u8, u8) {
		let byte1: u8 = (value & 0xFF) as u8;
		let byte2: u8 = ((value >> 8) & 0xFFu16) as u8;
		return (byte1, byte2);
	}
	
	fn little_endian_to_u16(i: u8, ii: u8) -> u16 {
		let byte1 = i as u16;
		let byte2 = ii as u16;
		return byte1 | (byte2 << 8);
	}
}