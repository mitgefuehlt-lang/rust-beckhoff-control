use crate::test_machine::TestMachine;
use crate::test_machine::api::TestMachineNamespace;
use smol::block_on;
use std::time::Instant;


use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::el2008::{EL2008, EL2008_IDENTITY_A, EL2008_IDENTITY_B, EL2008Port};
use ethercat_hal::devices::el1008::{EL1008, EL1008_IDENTITY_A, EL1008Port};
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::digital_input::DigitalInput;
use tracing::info;

//Imports For Wago
/*
use ethercat_hal::devices::wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354};
use ethercat_hal::devices::{EthercatDevice, downcast_device};
use smol::lock::RwLock;
use std::sync::Arc;
*/

impl MachineNewTrait for TestMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        info!("[TestMachine::new] Starting initialization...");
        
        // validate general stuff
        let device_identification = params
            .device_group
            .iter()
            .map(|device_identification| device_identification.clone())
            .collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                let err = anyhow::anyhow!(
                    "[{}::MachineNewTrait/TestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                );
                tracing::error!("{}", err);
                return Err(err);
            }
        };

        block_on(async {
            info!("[TestMachine::new] Acquiring EL1008 (Role 0)...");
            let el1008_res = get_ethercat_device::<EL1008>(hardware, params, 0, [EL1008_IDENTITY_A].to_vec()).await;
            
            let el1008 = match el1008_res {
                Ok(dev) => {
                    info!("[TestMachine::new] Successfully acquired EL1008");
                    dev.0
                },
                Err(e) => {
                    tracing::error!("[TestMachine::new] Failed to acquire EL1008: {:?}", e);
                    return Err(e);
                }
            };

            let di1 = DigitalInput::new(el1008.clone(), EL1008Port::DI1);
            let di2 = DigitalInput::new(el1008.clone(), EL1008Port::DI2);
            let di3 = DigitalInput::new(el1008.clone(), EL1008Port::DI3);
            let di4 = DigitalInput::new(el1008.clone(), EL1008Port::DI4);
            let di5 = DigitalInput::new(el1008.clone(), EL1008Port::DI5);
            let di6 = DigitalInput::new(el1008.clone(), EL1008Port::DI6);
            let di7 = DigitalInput::new(el1008.clone(), EL1008Port::DI7);
            let di8 = DigitalInput::new(el1008.clone(), EL1008Port::DI8);

            info!("[TestMachine::new] Acquiring EL2008 (Role 1)...");
            // Allow Identity A and B
            let el2008_res = get_ethercat_device::<EL2008>(hardware, params, 1, [EL2008_IDENTITY_A, EL2008_IDENTITY_B].to_vec()).await;
            
            let el2008 = match el2008_res {
                Ok(dev) => {
                    info!("[TestMachine::new] Successfully acquired EL2008");
                    dev.0
                },
                Err(e) => {
                    tracing::error!("[TestMachine::new] Failed to acquire EL2008: {:?}", e);
                    return Err(e);
                }
            };

            let do1 = DigitalOutput::new(el2008.clone(), EL2008Port::DO1);
            let do2 = DigitalOutput::new(el2008.clone(), EL2008Port::DO2);
            let do3 = DigitalOutput::new(el2008.clone(), EL2008Port::DO3);
            let do4 = DigitalOutput::new(el2008.clone(), EL2008Port::DO4);
            let do5 = DigitalOutput::new(el2008.clone(), EL2008Port::DO5);
            let do6 = DigitalOutput::new(el2008.clone(), EL2008Port::DO6);
            let do7 = DigitalOutput::new(el2008.clone(), EL2008Port::DO7);
            let do8 = DigitalOutput::new(el2008.clone(), EL2008Port::DO8);

            info!("[TestMachine::new] Initialization complete. Creating instance.");

            let (sender, receiver) = smol::channel::unbounded();
            let mut my_test = Self {
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: TestMachineNamespace {
                    namespace: params.namespace.clone(),
                },
                last_state_emit: Instant::now(),
                led_on: [false; 8],
                main_sender: params.main_thread_channel.clone(),
                douts: [do1, do2, do3, do4, do5, do6, do7, do8],
                dins: [di1, di2, di3, di4, di5, di6, di7, di8],
                blink_active: false,
                last_input_state: false,
                blink_timer: Instant::now(),
                blink_state: false,
            };
            my_test.emit_state();
            Ok(my_test)
        })
    }
}
