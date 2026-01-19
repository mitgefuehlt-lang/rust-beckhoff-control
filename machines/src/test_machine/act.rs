use super::TestMachine;
use crate::{MachineAct, MachineMessage};
use std::time::{Duration, Instant};

impl MachineAct for TestMachine {
    fn act(&mut self, now: Instant) {
        if let Ok(msg) = self.api_receiver.try_recv() {
            self.act_machine_message(msg);
        }

        // Logic: Input 0 toggles Blink State for Output 0
        if let Ok(current_input) = self.dins[0].get_value() {
            // Edge Detection: Rising Edge
            if current_input && !self.last_input_state {
                self.blink_active = !self.blink_active;
                tracing::info!("Toggle Blink: {}", self.blink_active);
                
                // Reset blink state when starting
                if self.blink_active {
                    self.blink_state = true;
                    self.blink_timer = now;
                }
            }
            self.last_input_state = current_input;
        }

        // Blinking Logic
        if self.blink_active {
            if now.duration_since(self.blink_timer) > Duration::from_millis(500) {
                self.blink_state = !self.blink_state;
                self.blink_timer = now;
            }
            self.douts[0].set(self.blink_state);
            self.led_on[0] = self.blink_state;
        } else {
            // If not blinking, force off (or keep as pass-through? Let's keep it as toggle ONLY for 0)
            self.douts[0].set(false);
            self.led_on[0] = false;
        }

        // Pass-through for other inputs (1-7)
        for i in 1..8 {
            if let Ok(val) = self.dins[i].get_value() {
                self.douts[i].set(val);
                self.led_on[i] = val;
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
