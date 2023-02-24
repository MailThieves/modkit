# Getting Started
The modkit package will start up a webserver when run. This web server does 2 things:

1. Expose a `/register` route
2. Expose a WebSocket server

## The `/register` route

The purpose of this route is to generate a websocket ID for a client. To use it, send a `GET` request to `/register`. For example, if the webserver is running on `localhost:3012`, you would send a `GET` request to `localhost:3012/register`. That request will return a WebSocket URL in a JSON package like the one below.

```JSON
{
	"url": "ws://127.0.0.1:3012/ws/761a39340f054e5aabaf347aaae6d838"
}
```

The long string at the end is an ID unique for each client. We go through this register process to ensure that every client has a unique ID.

Note that `127.0.0.1` is the same as `localhost`.

## Connecting to the WebSocket

Once you send the request above and you have a WebSocket url, you're ready to connect. There are several ways to connect, but here's an example in plain JavaScript.

```js
// Create WebSocket connection using the URL we got from registering
const socket = new WebSocket('ws://127.0.0.1:3012/ws/761a39340f054e5aabaf347aaae6d838');

socket.send("message here!");

// Set up a "listener" for incoming messages from the server
// This example listens for the text 'message', and logs to the console when that message is received
socket.addEventListener('message', (event) => {
    console.log('Message from server ', event.data);
});
```

# Events

The way communication happens between the interface and the modkit API is through the `Event` payload. Events are JSON objects that follow a certain format. Here's the definition of an `Event`.

```rust
struct Event {  
	/// The event type, needs to exactly match one of the options listed below
	kind: EventKind,  
	/// Which device this event references, if any  
	device: Option<DeviceType>,  
	/// The optional data bundle being sent
	data: Option<Bundle>  
}

enum EventKind {  
	// Interface -> API
	HealthCheck,
	PollDevice,
	// API -> Interface
	MailDelivered,
	MailPickedUp,
	DoorOpened,
	PollDeviceResult,
	Error
}

// The valid types of devices
enum DeviceType {
	Camera,
	Light,
	ContactSensor,
}
```
So events *must* have at least an event kind (a string matching one of the event kinds listed), and can optionally include a device and data bundle.

The event kind (`EventKind`) determines what the `Event` means. For example, the API sending an event to the interface with the `MailDelivered` kind means that mail has been delivered (go figure). The interface sending an event with the `PollDevice` kind is asking for data about a specific device, and expects a reponse.

Some event kinds are marked as `Interface -> API`, and some `API -> Interface`. The API will refuse requests that it's not expecting (ie. if the interface tries to tell the modkit API that mail was delivered, it will refuse because that doesn't make sense). 

## Examples

Here's some examples of `Event` based communication

### Health check
```json
Interface Sends:
{
	"kind": "HealthCheck"
}

API Returns:
{
	"kind": "HealthCheck",
	"timestamp": "2023-02-23 01:02:14.840327962 -06:00",
	"device": null,
	"data": null
}
```
### Poll Device
```json
Interface Sends:
{
	"kind": "PollDevice",
    "device": "ContactSensor"
}

API Returns:
{
	"kind": "PollDeviceResult",
	"timestamp": "2023-02-23 01:02:59.839505982 -06:00",
	"device": "ContactSensor",
	"data": {
	"ContactSensor": {
		"open": false
		}
	}
}
```

### Error
The `PollDevice` event requires you specify a device to poll. Here's what an error would look like.
```json
Interface Sends:
{
	"kind": "PollDevice"
	// there is no "device": "---" key here
}

API Returns:
{
	"kind": "Error",
	"timestamp": "2023-02-23 01:04:35.116551686 -06:00",
	"device": null,
	"data": {
	"Error": {
		"msg": "Please provide a device type to poll (\"device\": \"Camera\" for example)"
		}
	}
}
```