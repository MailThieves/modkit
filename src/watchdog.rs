use std::time::Duration;

use log::*;

use crate::drivers::contact_sensor::ContactSensor;
use crate::drivers::device::DeviceType;
use crate::server::Clients;
use crate::store::Store;
use crate::{model::*, server};

// 1. Watch for door opening
// 2. Trigger the light
// 3. Trigger the camera
// 4. Send an event to the clients
// 5. write event in the db
// 6. profit?
//
// The events that could be sent from this loop:
// 1. MailDelivered
// 2. MailPickedUp
// 3. DoorOpened
pub async fn watch(clients: &Clients) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running the watchdog");
    let store = Store::connect().await?;

    // First, set up our door sensor
    let mut door_sensor = ContactSensor::new();

    let mut event_queue: Vec<Event> = Vec::new();

    loop {
        // if the door sensor changes
        // (this calls poll() and updated the internal state)
        if door_sensor.changed().unwrap_or(false) {
            let is_open: bool = door_sensor.is_open();

            // Queue up an event to send with the door state.
            // We always send an event when the door opens or closes.
            // They don't have to do anything with it, but it's there.
            // We don't want to call poll_device() here because we already did above
            event_queue.push(Event::new(
                EventKind::DoorOpened,
                Some(DeviceType::ContactSensor),
                Some(Bundle::ContactSensor { open: is_open }),
            ));

            debug!("Door is open? {is_open}");

            // When the door changes to closed (ie. someone opens the box then
            // closes it, mail delivered or picked up)
            if !is_open {
                trace!("Door just closed, taking a picture!");

                // Make a new event with the associated Camera type
                let mut new_image_event =
                    Event::new(EventKind::NewImage, Some(DeviceType::Camera), None);
                // Call poll_device, which will take a picture and store the data bundle on itself
                let new_image_bundle = new_image_event.poll_device();
                // Then queue it up to be sent
                event_queue.push(new_image_event);

                // Old code, might not need
                // match camera::capture_into("./img") {
                //     Ok(file_path) => {
                //         trace!("Captured successfully, placed into `./img`");
                //         event_queue.push(Event::new(
                //             EventKind::NewImage,
                //             None,
                //             Some(Bundle::Camera { file_name: () }),
                //         ))
                //     }
                //     Err(e) => error!("{e}"),
                // };

                // Here we should check if mail was delievered or picked up.
                // At the least we can use the DB to determine that.
                // Best case scenario is to use image processing
                info!("Queueing up a MailDelivered Event");
                event_queue.push(Event::new(EventKind::MailDelivered, None, None));

                // Also send an image capture event
            }
        }

        // For all events in the queue, send them to all clients
        // and also write it to the db
        for event in event_queue {
            trace!("Sending event {} to clients", event.kind());
            trace!("{:#?}", event);
            server::ws::send_to_clients(&event, &clients).await;
            store.write_event(event).await?;
        }

        event_queue = Vec::new();
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
