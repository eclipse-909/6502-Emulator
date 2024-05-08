use {
	std::cmp::Ordering,
	crate::hardware::hardware::Hardware
};

///A type of hardware that can interrupt the CPU.
pub trait Interrupt: Hardware {
	///All devices are expected to have an output buffer register
	fn get_out_buf(&self) -> u8;
	///All devices are expected to have an InterruptSpecs
	fn get_interrupt_specs(&self) -> &InterruptSpecs;
}

///Common properties of an interrupt device.
#[derive(Clone)]
pub struct InterruptSpecs {
	pub iqr: u8,
	pub priority: u8,
	pub name: String
}

impl InterruptSpecs {
	pub fn new(iqr: u8, priority: u8, name: &str) -> Self {
		Self {
			iqr,
			priority,
			name: String::from(name)
		}
	}
}
//Traits needed for BinaryHeap comparison. All comparisons check the value of the priority and nothing else.
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