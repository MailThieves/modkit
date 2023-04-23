use std::time::Duration;

use log::*;

use crate::drivers::contact_sensor::ContactSensor;
use crate::drivers::device::DeviceType;
use crate::server::Clients;
use crate::store::Store;
use crate::{model::*, server};

/// Runs a continuous loop that watches for the door state changing.
/// If the state changes:
///     1. Immediately send an event that the door opened or closed (skip the queue)
///     2. If the door opened, record a 5 second video and send a PollDeviceResult (Camera) when
///        done
///             Unfortunately this blocks, we can't do it async
///     3. If the door closed, send either a MailDelivered or MailPickedUp event
pub async fn watch(clients: &Clients) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running the watchdog");
    let store = Store::connect().await?;

    // Set up our door sensor
    let mut door_sensor = ContactSensor::new();

    // Make an event queue
    let mut event_queue: Vec<Event> = Vec::new();

    loop {
        // if the door sensor changes
        // (changed() calls poll() and updates the internal state)
        if door_sensor.changed().unwrap_or(false) {
            let is_open: bool = door_sensor.is_open();

            // Skip the queue and just send an event immediately
            // We want to skip the queue because right after this, we might record
            // a video. Recording a video will block for 6 seconds. If we queued up,
            // it would only send the door event after waiting for the recording, and the
            // interface may get several events at once which isn't idead'
            let opened_event = Event::new(
                EventKind::DoorOpened,
                Some(DeviceType::ContactSensor),
                Some(Bundle::ContactSensor { open: is_open }),
            );
            server::ws::send_to_clients(&opened_event, &clients).await;

            // When the door opens, take a video and send that event
            if is_open {
                trace!("Door opened, taking a video (after 1 second delay)");
                // Make a new event with the associated Camera type
                let mut new_video_event =
                    Event::new(EventKind::PollDeviceResult, Some(DeviceType::Camera), None);
                // Call poll_device, which will take a video and store the data bundle on itself
                new_video_event.poll_device()?;
                // Then queue it up to be sent
                event_queue.push(new_video_event);
            }

            // When the door changes to closed (ie. someone opens the box then
            // closes it, mail delivered or picked up)
            if !is_open {
                // Normally we should use image processing or something to determine if the mail is picked up,
                // but I don't have time for that now. This just switches between the statuses.
                if let Ok(mail_status) = store.get_mail_status().await {
                    match mail_status.kind() {
                        EventKind::MailDelivered => {
                            // If the last status (ie. last time the door opened) was a delivery,
                            // then mail is being picked up
                            info!("Queueing up a MailPickedUp Event");
                            event_queue.push(Event::new(EventKind::MailPickedUp, None, None));
                        }
                        EventKind::MailPickedUp => {
                            info!("Queueing up a MailDelivered Event");
                            event_queue.push(Event::new(EventKind::MailDelivered, None, None));
                        }
                        _ => {}
                    }
                } else {
                    event_queue.push(Event::new(EventKind::MailDelivered, None, None));
                }
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
