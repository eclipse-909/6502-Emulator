use {
	std::cmp::Ordering,
	crate::hardware::hardware::Hardware
};

/**A type of hardware that can interrupt the CPU.*/
pub trait Interrupt: Hardware {
	fn get_out_buf(&self) -> u8;
	fn get_interrupt_specs(&self) -> &InterruptSpecs;
}

/**Common properties of an interrupt device.*/
pub struct InterruptSpecs {//TODO make this impl Ord for BinaryHeap comparisons
	pub iqr: u8,
	pub priority: u8,
	pub name: String
}

impl InterruptSpecs {
	pub fn new(iqr: u8, priority: u8, name: &str) -> Self {
		return Self {
			iqr,
			priority,
			name: String::from(name)
		};
	}
}
impl Clone for InterruptSpecs {
	fn clone(&self) -> Self {
		Self {
			iqr: self.iqr,
			priority: self.priority,
			name: self.name.clone()
		}
	}
}
impl Eq for InterruptSpecs {}
impl PartialEq<Self> for InterruptSpecs {
	fn eq(&self, other: &Self) -> bool {PartialEq::eq(&self.priority, &other.priority)}
}
impl PartialOrd<Self> for InterruptSpecs {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {PartialOrd::partial_cmp(&self.priority, &other.priority)}
}
impl Ord for InterruptSpecs {
	fn cmp(&self, other: &Self) -> Ordering {Ord::cmp(&self.priority, &other.priority)}
}