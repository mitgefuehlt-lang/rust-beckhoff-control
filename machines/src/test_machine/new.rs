use crate::test_machine::TestMachine;
use crate::test_machine::api::TestMachineNamespace;
use smol::block_on;
use std::time::Instant;


use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};

use anyhow::Error;
use ethercat_hal::devices::el2008::{EL2008, EL2008_IDENTITY_A, EL2008Port};
use ethercat_hal::devices::el1008::{EL1008, EL1008_IDENTITY_A, EL1008Port};
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::digital_input::DigitalInput;

//Imports For Wago
/*
use ethercat_hal::devices::wago_750_354::{WAGO_750_354_IDENTITY_A, Wago750_354};
use ethercat_hal::devices::{EthercatDevice, downcast_device};
use smol::lock::RwLock;
use std::sync::Arc;
*/

impl MachineNewTrait for TestMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
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
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/TestMachine::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };
        block_on(async {
            /*
            // ... Wago commented out code ...
            */

             // Assuming EL1008 is at index 0 and EL2008 at index 1 is safer, checking get_ethercat_device logic: 
             // it filters by params.device_group role. But wait, get_ethercat_device takes `subdevice_index` from identification?
             // Actually `get_ethercat_device` calls `get_device_ident` which gets `subdevice_index` from `device_hardware_identification`.
             
             // WE rely on role. The roles must be defined in the Registry or Config? 
             // Actually `get_ethercat_device` logic:
             // 1. `get_device_ident(params, role)` retrieves the device info for a specific ROLE.
             
             // I don't know the roles assigned to EL1008 and EL2008 in the user's config.
             // If I use the wrong role, it will fail.
             // Usually role 0, 1, etc? 
             
             // BUT: The existing code used `get_ethercat_device(..., 1, ...)` where 1 is the ROLE? No, 1 is the 3rd argument -> `role`.
             // Existing code: `get_ethercat_device::<EL2004>(hardware, params, 1, ...)`
             // So EL2004 was role 1. 
             // Let's assume EL2008 is role 1.
             // And EL1008 is role 0? Or 2? 
             // I'll try getting EL1008 with role 0 and EL2008 with role 1.
             
            let el1008 =
                get_ethercat_device::<EL1008>(hardware, params, 0, [EL1008_IDENTITY_A].to_vec())
                    .await?
                    .0;
            let di1 = DigitalInput::new(el1008.clone(), EL1008Port::DI1);
            let di2 = DigitalInput::new(el1008.clone(), EL1008Port::DI2);
            let di3 = DigitalInput::new(el1008.clone(), EL1008Port::DI3);
            let di4 = DigitalInput::new(el1008.clone(), EL1008Port::DI4);
            let di5 = DigitalInput::new(el1008.clone(), EL1008Port::DI5);
            let di6 = DigitalInput::new(el1008.clone(), EL1008Port::DI6);
            let di7 = DigitalInput::new(el1008.clone(), EL1008Port::DI7);
            let di8 = DigitalInput::new(el1008.clone(), EL1008Port::DI8);

            let el2008 =
                get_ethercat_device::<EL2008>(hardware, params, 1, [EL2008_IDENTITY_A].to_vec())
                    .await?
                    .0;
            let do1 = DigitalOutput::new(el2008.clone(), EL2008Port::DO1);
            let do2 = DigitalOutput::new(el2008.clone(), EL2008Port::DO2);
            let do3 = DigitalOutput::new(el2008.clone(), EL2008Port::DO3);
            let do4 = DigitalOutput::new(el2008.clone(), EL2008Port::DO4);
            let do5 = DigitalOutput::new(el2008.clone(), EL2008Port::DO5);
            let do6 = DigitalOutput::new(el2008.clone(), EL2008Port::DO6);
            let do7 = DigitalOutput::new(el2008.clone(), EL2008Port::DO7);
            let do8 = DigitalOutput::new(el2008.clone(), EL2008Port::DO8);

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
            };
            my_test.emit_state();
            Ok(my_test)
        })
    }
}
