use crate::pdo::el252x::{EncControl, EncStatus, PtoControl, PtoStatus, PtoTarget};

/// Trait for pulse train output devices
pub trait PulseTrainOutputDevice<Port> {
    fn set_output(&mut self, port: Port, value: PulseTrainOutputOutput);
    fn get_output(&self, port: Port) -> PulseTrainOutputOutput;
    fn get_input(&self, port: Port) -> PulseTrainOutputInput;
}

#[derive(Debug, Clone)]
pub struct PulseTrainOutputInput {
    pub select_end_counter: bool,
    pub ramp_active: bool,
    pub input_t: bool,
    pub input_z: bool,
    pub error: bool,
    pub sync_error: bool,
    pub txpdo_toggle: bool,
    pub set_counter_done: bool,
    pub counter_underflow: bool,
    pub counter_overflow: bool,
    pub counter_value: u32,
}

#[derive(Debug, Clone)]
pub struct PulseTrainOutputOutput {
    pub disable_ramp: bool,
    pub frequency_select: bool,
    pub go_counter: bool,
    pub frequency_value: i32,
    pub target_counter_value: u32,
    pub set_counter: bool,
    pub set_counter_value: u32,
}
