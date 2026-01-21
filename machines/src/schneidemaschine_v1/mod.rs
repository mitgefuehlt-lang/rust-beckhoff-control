use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::schneidemaschine_v1::api::{SchneideMaschineV1Events, StateEvent};
use crate::{AsyncThreadMessage, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::digital_output::DigitalOutput;
use smol::channel::{Receiver, Sender};
use std::time::Instant;
pub mod act;
pub mod api;
pub mod new;
use crate::schneidemaschine_v1::api::SchneideMaschineV1Namespace;
use crate::{MACHINE_SCHNEIDEMASCHINE_V1, VENDOR_KAILAR};
use ethercat_hal::devices::el2522::EL2522;
use smol::lock::RwLock;
use std::sync::Arc;

#[derive(Debug)]
pub struct SchneideMaschineV1 {
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: SchneideMaschineV1Namespace,
    pub last_state_emit: Instant,
    pub led_on: [bool; 8],
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub douts: [DigitalOutput; 8],
    pub dins: [DigitalInput; 8],
    // Blink Logic State
    pub blink_active: bool,
    pub last_input_state: bool,
    pub blink_timer: Instant,
    pub blink_state: bool,
    // Motor Control
    pub pto: Arc<RwLock<EL2522>>,
    pub motor_target_mm: f64,
    pub motor_speed_mm_s: f64,
    pub motor_running: bool,
    pub motor_was_running: bool,
    pub last_button_state: bool,
    pub start_time: Instant,
    // PTO Status Cache
    pub pto_counter_value: u32,
    pub pto_error: bool,
    pub pto_ramp_active: bool,
}

impl Machine for SchneideMaschineV1 {
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> {
        self.main_sender.clone()
    }
}
impl SchneideMaschineV1 {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_KAILAR,
        machine: MACHINE_SCHNEIDEMASCHINE_V1,
    };
}

impl SchneideMaschineV1 {
    pub fn emit_state(&mut self) {
        let event = StateEvent {
            led_on: self.led_on,
            motor_running: self.motor_running,
            motor_pos: self.pto_counter_value,
            motor_target: (self.motor_target_mm * 20.0) as u32,
            motor_freq: (self.motor_speed_mm_s * 20.0 * 100.0) as i32,
            motor_error: self.pto_error,
            motor_ramp_active: self.pto_ramp_active,
        }
        .build();

        self.namespace.emit(SchneideMaschineV1Events::State(event));
    }

    /// Set the state of a specific LED
    pub fn set_led(&mut self, index: usize, on: bool) {
        if index < self.led_on.len() {
            self.led_on[index] = on;
            self.douts[index].set(on);
            self.emit_state();
        }
    }

    /// Set all LEDs at once
    pub fn set_all_leds(&mut self, on: bool) {
        self.led_on = [on; 8];
        for dout in self.douts.iter() {
            dout.set(on);
        }
        self.emit_state();
    }
}
