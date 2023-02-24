use std::thread::sleep;
use std::time::Duration;
use log::*;

use crate::drivers::contact_sensor::ContactSensor;
use crate::drivers::device::{Device, DeviceType, Bundle};
use crate::ws::event::{Event, EventKind};
use crate::ws::ws::Clients;

pub async fn watch(clients: &mut Clients) {
    
    let door_sensor = ContactSensor::new("Door Sensor", 0, "sensor.txt");
    let mut door_sensor_old_state = match door_sensor.poll() {
        Ok(Bundle::ContactSensor { open }) => open,
        _ => panic!()
    };

    // If this is Some(event) then it will be sent at the end of the loop and reset
    let mut event: Option<Event> = None;

    loop {

        // it's been a while since I wrote this but I think what's happening is that
        // This continually checks for changes in state. When a change is found, generate an event and
        // send it at the end of the loop.
        //
        // We may need to implement a queue of events if there are multiple at once.
        // 
        // TODO: if the contact sensor continually errors, will this always be false?
        if door_sensor.is_active().unwrap_or(false) != door_sensor_old_state {
            // TODO: is_active() calls poll() anyway, you should only call it once
            let bundle = match door_sensor.poll() {
                Ok(bundle) => {
                    if let Bundle::ContactSensor { open } = bundle {
                        door_sensor_old_state = open;
                    }
                    bundle
                },
                Err(_) => todo!(),
            };
            event = Some(Event::new(EventKind::DoorOpened, Some(DeviceType::ContactSensor), Some(bundle)));
        }

        if let Some(ev) = event {
            let lock = clients.lock().await;
            for (id, client) in lock.iter() {
                info!("Sending to client {id}");
                if let Some(sender) = &client.sender {
                    sender.send(Ok(ev.clone().to_msg())).expect("Couldn't send message to client");
                }
            }
            drop(lock);
        }
        event = None;
        sleep(Duration::from_secs(1));
    }
}
