///The clock will impl ClockListener, and its pulse function should call the pulse function for all other ClockListeners.
pub trait ClockListener {
	///Called on each clock cycle if registered.
	fn pulse(&mut self);
}