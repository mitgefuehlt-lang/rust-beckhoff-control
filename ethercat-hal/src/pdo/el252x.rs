use super::{RxPdoObject, TxPdoObject};
use bitvec::{field::BitField, order::Lsb0, slice::BitSlice};
use ethercat_hal_derive::PdoObject;

/// PDO Object for EL252x devices
///
/// "PTO Status" contains status information about the pulse train output.
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 16)]
pub struct PtoStatus {
    pub select_end_counter: bool,
    pub ramp_active: bool,
    pub input_t: bool,
    pub input_z: bool,
    pub error: bool,
    pub sync_error: bool,
    pub txpdo_toggle: bool,
}

impl TxPdoObject for PtoStatus {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // only read other values if txpdo_toggle is true
        self.txpdo_toggle = bits[8 + 7];
        if !self.txpdo_toggle {
            return;
        }

        self.select_end_counter = bits[0];
        self.ramp_active = bits[1];
        self.input_t = bits[4];
        self.input_z = bits[5];
        self.error = bits[6];

        self.sync_error = bits[8 + 5];
    }
}

/// PDO Object for EL252x devices
///
/// "Encoder Status" contains the encoder status information.
#[derive(Debug, Clone, Default, PdoObject, PartialEq, Eq)]
#[pdo_object(bits = 48)]
pub struct EncStatus {
    /// Acknowedges the set counter command of the last cycle
    pub set_counter_done: bool,

    /// If the real positon is less than the u32 position and the counter underflowed
    pub counter_underflow: bool,

    /// If the real positon is greater than the u32 position and the counter overflowed
    pub counter_overflow: bool,

    pub sync_error: bool,

    /// If the PDO objects data has changed since the last read
    pub txpdo_toggle: bool,

    /// The counted position/pulses by the encoder
    pub counter_value: u32,
}

impl TxPdoObject for EncStatus {
    fn read(&mut self, bits: &BitSlice<u8, Lsb0>) {
        // only read other values if txpdo_toggle is true
        self.txpdo_toggle = bits[8 + 7];
        if !self.txpdo_toggle {
            return;
        }

        self.set_counter_done = bits[2];
        self.counter_underflow = bits[3];
        self.counter_overflow = bits[4];
        self.sync_error = bits[8 + 5];
        self.counter_value = bits[16..16 + 32].load_le();
    }
}

/// PDO Object for EL252x devices
///
/// "PTO Control" is used to control the pulse train output.
#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 32)]
pub struct PtoControl {
    pub go_counter: bool,
    pub stop_counter: bool,
    pub set_counter: bool,
    pub reset_counter: bool,
    pub select_end_counter: bool,
    pub input_z_logic: bool,
    pub reset: bool,
    pub input_t_logic: bool,
    pub disable_ramp: bool,
    pub frequency_select: bool,
    pub control_toggle: bool,

    /// Pulse frequency value in Hz (actually 0.01 Hz units)
    pub frequency_value: i32,
}

impl RxPdoObject for PtoControl {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer.set(0, self.go_counter);
        buffer.set(1, self.stop_counter);
        buffer.set(2, self.set_counter);
        buffer.set(3, self.reset_counter);
        buffer.set(4, self.select_end_counter);
        buffer.set(5, self.input_z_logic);
        buffer.set(6, self.reset);
        buffer.set(7, self.input_t_logic);
        buffer.set(8, self.disable_ramp);
        buffer.set(9, self.frequency_select);
        buffer.set(15, self.control_toggle);

        buffer[16..16 + 16].store_le(self.frequency_value);
    }
}

/// PDO Object for EL252x devices
///
/// "PTO Target" is used to set the target position of the pulse train output.
#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 32)]
pub struct PtoTarget {
    /// Target position in pulses
    ///
    /// Target of the [`EncStatus::counter_value`] field
    pub target_counter_value: u32,
}

impl RxPdoObject for PtoTarget {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer[0..32].store_le(self.target_counter_value);
    }
}

/// PDO Object for EL252x devices
///
/// "Encoder Control" is used to control the encoder.
#[derive(Debug, Clone, Default, PdoObject)]
#[pdo_object(bits = 48)]
pub struct EncControl {
    /// Set to `true` when wanting to override the encoder position
    pub set_counter: bool,
    /// Value to set the encoder to
    pub set_counter_value: u32,
}

impl RxPdoObject for EncControl {
    fn write(&self, buffer: &mut BitSlice<u8, Lsb0>) {
        buffer.set(2, self.set_counter);

        buffer[16..16 + 32].store_le(self.set_counter_value);
    }
}
