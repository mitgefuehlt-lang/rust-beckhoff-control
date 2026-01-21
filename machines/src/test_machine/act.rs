use super::TestMachine;
use crate::{MachineAct, MachineMessage};
use ethercat_hal::devices::el2522::EL2522Port;
use ethercat_hal::io::pulse_train_output::{PulseTrainOutputDevice, PulseTrainOutputOutput};
use std::time::{Duration, Instant};

impl MachineAct for TestMachine {
    fn act(&mut self, now: Instant) {
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Direct Mapping: Inputs pass through to Outputs
        for i in 0..8 {
            if let Ok(val) = self.dins[i].get_value() {
                self.douts[i].set(val);
                self.led_on[i] = val;

                // --- Button Logic (Input 0) ---
                // Motor logic update for physical trigger
                if i == 0 {
                    if val && !self.last_button_state {
                        // Rising Edge detected
                        self.motor_running = !self.motor_running;
                        if self.motor_running {
                            self.motor_target_mm = 500.0; // Large target for continuous feel
                            self.motor_speed_mm_s = 2.0;  // Very slow (2mm/s = 40 pulses/s)
                        } else {
                            self.motor_speed_mm_s = 0.0; // Stop
                        }
                    }
                    self.last_button_state = val;
                }
            }
        }

        // --- Motor Control Logic (Runs every cycle) ---
        {
            // Conversion: 200 pulses/rev, 10mm/rev -> 20 pulses per mm
            let target_pulses = (self.motor_target_mm * 20.0) as u32;
            let frequency_hz = (self.motor_speed_mm_s * 20.0) as i32;

            let mut pto = smol::block_on(self.pto.write());
            let current_output = pto.get_output(EL2522Port::PTO2);

            // Reset counter to 0 on motor start (rising edge of motor_running)
            let mut set_counter = false;
            if self.motor_running && !self.motor_was_running {
                set_counter = true;
                tracing::info!(
                    "[TestMachine] Motor Start: Resetting counter to 0, target: {} pulses",
                    target_pulses
                );
            }
            self.motor_was_running = self.motor_running;

            // Only update if target, frequency, go_counter or set_counter state changed
            if current_output.target_counter_value != target_pulses
                || current_output.frequency_value != frequency_hz
                || current_output.go_counter != self.motor_running
                || set_counter
            {
                pto.set_output(
                    EL2522Port::PTO2,
                    PulseTrainOutputOutput {
                        frequency_value: frequency_hz,
                        target_counter_value: target_pulses,
                        disble_ramp: false,
                        frequency_select: true,
                        go_counter: self.motor_running,
                        set_counter,
                        set_counter_value: 0,
                        ..current_output
                    },
                );
            }
        }

        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::ConnectToMachine(_machine_connection) => {
                // Does not connect to any Machine; do nothing
            }
            MachineMessage::DisconnectMachine(_machine_connection) => {
                // Does not connect to any Machine; do nothing
            }
        }
    }
}
