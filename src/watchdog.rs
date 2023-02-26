use log::*;
use std::time::Duration;

use crate::drivers::contact_sensor::ContactSensor;
use crate::drivers::device::{Device, DeviceType};
use crate::store::Store;
use crate::{model::*, ws};
use crate::ws::ws::Clients;

// 1. Watch for door opening
// 2. Trigger the light
// 3. Trigger the camera
// 4. Send an event to the clients
// 5. write event in the db
// 6. profit?
//
// The events that could be sent from this loop:
// 1. MailDelivered (if determinable)
// 2. MailPickedUp  (if determinable)
// 3. DoorOpened    (if undeterminable)
pub async fn watch(clients: &Clients) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running the watchdog");
    let store = Store::connect().await?;

    // First, set up our door sensor
    let mut door_sensor = ContactSensor::new("Door Sensor", 0, "sensor.txt");

    let mut event_queue: Vec<Event> = Vec::new();

    loop {

        // if the door sensor changes
        if door_sensor.changed().unwrap_or(false) {
            // Call the on_activate() function
            door_sensor.on_activate().unwrap();
            // Get a copy of the state
            let bundle = door_sensor.state().unwrap().clone();
            // Queue up an event to send with the door state
            event_queue.push(Event::new(
                EventKind::DoorOpened,
                Some(DeviceType::ContactSensor),
                Some(bundle),
            ));

            // Here we should also determine if mail is delivered or not. This will require the camera.
        }


        // For all events in the queue, send them to all clients
        // and also write it to the db
        for event in event_queue {
            ws::send_to_clients(&event, &clients).await;
            store.write_event(event).await?;
        }

        event_queue = Vec::new();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
