/**Callback handler which implements a pulse function, which is invoked for each cycle of the clock.*/
pub trait ClockListener {
	/**Called on each clock cycle if registered.*/
	fn pulse(&mut self);
}