use {
	crate::hardware::{
		hardware::{Hardware, HardwareSpecs},
		imp::clock_listener::ClockListener,
		memory::{Memory, MemEvent, N_WAYS},
		cache::Cache
	},
	tokio::sync::mpsc::{Sender, Receiver}
};

pub struct Mmu {
	specs: HardwareSpecs,
	pub cache: Cache
}

impl Hardware for Mmu {
	fn get_specs(&self) -> &HardwareSpecs {&self.specs}
}

impl Mmu {
	pub fn new(channels: [(Sender<MemEvent>, Receiver<MemEvent>); N_WAYS as usize]) -> Self {
		let mmu: Self = Self {
			specs: HardwareSpecs::new("MMU"),
			cache: Cache::new(channels)
		};
		mmu.log("Created");
		mmu
	}
	
	//Startup functions that are called before the clock starts pulsing
	///Takes a &\[u8] and flashes it into RAM. The first value in the slice is stored at start_addr.
	pub fn static_load(&mut self, memory: &mut [Memory; N_WAYS as usize], values: &[u8], start_addr: u16) {
		let mut iter = values.iter().enumerate();
		let Some((mut i, mut val)) = iter.next() else {return;};
		loop {
			self.cache.write(start_addr + i as u16, *val);
			memory.iter_mut().for_each(|mem| {mem.pulse();});
			let Some((next_i, next_val)) = iter.next() else {return;};//advance the iterator if memory action was successful
			i = next_i;
			val = next_val;
		}
	}
	///Logs the values at each memory address in the range start_addr..end_addr
	pub fn memory_dump(&mut self, memory: &mut [Memory; N_WAYS as usize], start_addr: u16, end_addr: u16) {
		let mut iter = start_addr..end_addr;
		let Some(mut i) = iter.next() else {return;};
		loop {
			if let Ok(Some(mdr)) = self.cache.read(i) {
				self.log(format!("Address 0x{:04x}: | 0x{:02X}", i, mdr).as_str());
				let Some(next) = iter.next() else {return;};//advance the iterator if memory action was successful
				i = next;
			} else {
				memory.iter_mut().for_each(|mem| {mem.pulse();});//force memory to do its thing
			}
		}
	}
}