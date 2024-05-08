use {
	crate::hardware::{
		hardware::{Hardware, HardwareSpecs},
		memory::{MemEvent, N_WAYS}
	},
	std::collections::HashMap,
	tokio::sync::mpsc::{Receiver, Sender, error::TryRecvError}
};

const NUM_LINES: u8 = 16;
///There should be no zeros in this number
pub const INDEX_MASK: u8 = 0b111;

pub struct Cache {
	specs: HardwareSpecs,
	lines: HashMap<u16, CacheLine>,
	pub memory: [(Sender<MemEvent>, Receiver<MemEvent>); N_WAYS as usize],
	pub cache_hits: u128,
	pub cache_accesses: u128
}

impl Hardware for Cache {
	fn get_specs(&self) -> &HardwareSpecs {&self.specs}
}

impl Cache {
	pub fn new(channels: [(Sender<MemEvent>, Receiver<MemEvent>); N_WAYS as usize]) -> Self {
		let cache: Self = Self {
			specs: HardwareSpecs::new("Cache"),
			lines: HashMap::with_capacity(NUM_LINES as usize),
			memory: channels,
			cache_hits: 0,
			cache_accesses: 0
		};
		cache.log("Created");
		cache
	}
	
	/**Returns Ok(Some(u8)) if cache hit, Ok(None) if cache miss, or Err(()) if it can't perform a read on this cycle.
	If the value is not returned, this function must be called again in a future cycle until it is. The value can either be
	returned in a cache hit, or when the value is returned from memory.*/
	pub fn read(&mut self, address: u16) -> Result<Option<u8>,()> {
		self.cache_accesses += 1;
		let index: usize = (address & INDEX_MASK as u16) as usize;
		let tag: u16 = (address & !INDEX_MASK as u16) / N_WAYS as u16;
		let mut val: Option<u8> = None;
		if let Some(line) = self.lines.get_mut(&tag) {
			//cache hit - immediately return value
			val = Some(line.data[index]);
			let prev_age: u8 = line.status & INDEX_MASK;
			line.status &= !INDEX_MASK;//set age to 0
			for (t, l) in self.lines.iter_mut() {
				if *t != tag && (l.status & INDEX_MASK) < prev_age {
					l.status += 1;
				}
			}
			//clear cache buffer and discard the value
			self.memory.iter_mut().for_each(|(_, rx)| {rx.try_recv().ok();});
			self.cache_hits += 1;
		} else {
			//cache miss
			let mut data: [u8; N_WAYS as usize] = [0x00; N_WAYS as usize];
			//read cache buffer
			let mut responses: u8 = 0b00;
			for (i, (_, rx)) in self.memory.iter_mut().enumerate() {
				match rx.try_recv() {
					Ok(MemEvent::MemReadResponse{mdr}) => {
						data[i] = mdr;
						responses |= 0b01;
					}
					Ok(MemEvent::MemWriteResponse) => {
						responses |= 0b10;
					}
					Ok(_) => {
						panic!("Received invalid response from memory.");
					}
					Err(TryRecvError::Empty) => {}
					Err(TryRecvError::Disconnected) => {
						panic!("Memory tx disconnected.");
					}
				}
			}
			match responses {
				0b00 | 0b10 => {
					//request to read from memory
					self.memory.iter_mut().for_each(|(tx, _)| {tx.try_send(MemEvent::MemReadRequest{mar: tag}).expect("Memory receiver buffer full.");});
					return Ok(None);
				}
				0b11 => {
					return Err(())//cannot read and write to memory simultaneously
				}
				_ => {}
			}
			//the memory has responded with the data -
			let new_line: CacheLine = CacheLine::new(data);
			let mut prev_age: u8 = N_WAYS;
			if self.lines.len() == NUM_LINES as usize {//if cache has filled all lines
				//remove the oldest cache line
				let Some((oldest_tag, _)) = self.lines.iter().max_by(|(_, line1), (_, line2)| {(line1.status & INDEX_MASK).cmp(&(line2.status & INDEX_MASK))}) else {panic!("Cache lines HashMap was empty and not empty at the same time.");};
				let oldest_tag: u16 = oldest_tag.to_owned();
				let removed_line: CacheLine = self.lines.remove(&oldest_tag).expect("Cache Line exists and doesn't exist at the same time.");
				prev_age = removed_line.status & INDEX_MASK;
				//write it back if dirty
				if removed_line.status & !INDEX_MASK > 0 {
					self.memory.iter_mut().enumerate().for_each(|(i, (tx, _))| {
						tx.try_send(MemEvent::MemWriteRequest { mar: oldest_tag, mdr: removed_line.data[i] }).expect("Memory receiver buffer full.");
					});
				}
			}
			for (_, l) in self.lines.iter_mut() {
				if (l.status & INDEX_MASK) < prev_age {
					l.status += 1;
				}
			}
			self.lines.insert(tag, new_line);
			if let Some(line) = self.lines.get(&tag) {
				val = Some(line.data[index]);
			}
		}
		Ok(val)
	}
	
	///Whether a write operation is successful must be handled by the caller. Returns true if writes to cache, false if writes to memory.
	pub fn write(&mut self, address: u16, value: u8) -> bool {
		self.cache_accesses += 1;
		//clear cache buffer
		for (_, rx) in self.memory.iter_mut() {
			match rx.try_recv() {
				Ok(MemEvent::MemReadResponse{..}) => {
					panic!("MemReadResponse was not handled");
				}
				Err(TryRecvError::Disconnected) => {
					panic!("Memory tx disconnected.");
				}
				Ok(MemEvent::MemWriteResponse) | Err(TryRecvError::Empty) => {}
				Ok(_) => {
					panic!("Cache received invalid value.");
				}
			}
		}
		let index: usize = (address & INDEX_MASK as u16) as usize;
		let tag: u16 = (address & !INDEX_MASK as u16) / N_WAYS as u16;
		if let Some(line) = self.lines.get_mut(&tag) {
			//cache hit
			line.data[index] = value;
			let prev_age: u8 = line.status & INDEX_MASK;
			line.status &= !INDEX_MASK;//set age to 0
			line.status |= 0b1000_0000;//set dirty flag
			for (t, l) in self.lines.iter_mut() {
				if *t != tag && (l.status & INDEX_MASK) < prev_age {
					l.status += 1;
				}
			}
			self.cache_hits += 1;
			true
		} else {
			//cache miss
			self.memory[index].0.try_send(MemEvent::MemWriteRequest {mar: tag, mdr: value}).expect("Memory receiver buffer full.");
			false
		}
	}
}

struct CacheLine {
	///0bD000_0AAA - D = dirty, A = age
	status: u8,
	data: [u8; N_WAYS as usize]
}

impl CacheLine {
	fn new(data: [u8; N_WAYS as usize]) -> Self {Self {status: 0x00, data}}
}