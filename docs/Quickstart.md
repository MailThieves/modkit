# About
This package is the software that runs in the mailbox (the "modkit"). It will start a webserver and listen for web traffic related to the mailbox. See the [[Event API]] documentation for specific info on how that works.

# Running

This repository is a Rust package (called a "crate"). Rust is a compiled language. You need either the Rust compiler to compile this crate and run it, **or** you can use a precompiled binary.

## Precompiled Binaries
I've put some precompiled binaries in the `bin/` folder. These are platform specific; use the one related to your operating system. Note that the platform I compile on may be slightly different from the one you use, which may break these. If that's the case, you'll need to compile from scratch. 

Otherwise, just run a binary in the `bin/` folder

```
# Windows
# An example, your location will be different

C:\Users\Luke Sweeney\code\modkit> bin\modkit_windows.exe
[INFO  modkit::ws] Running the WebSocket server
[INFO  warp::server] Server::run; addr=127.0.0.1:3012
[INFO  warp::server] listening on http://127.0.0.1:3012
```

```
# Linux
# An example, your location will be different

~/code/modkit $ bin/modkit
[INFO  modkit::ws] Running the WebSocket server
[INFO  warp::server] Server::run; addr=127.0.0.1:3012
[INFO  warp::server] listening on http://127.0.0.1:3012
```

After running the executable, you should see some server output and it should remain running. That means it's working!

## Compiling from Scratch
To compile from scratch, you'll need the rust compiler. Luckily, this is really easy to install. Go to https://rustup.rs/ and follow the instructions there.

Rust will install a few tools:
* `rustc` - The bare Rust compiler, we almost never use this directly
* `rustup` - a tool for managing which version of rust you have installed. We don't use this often
* `cargo` - Rust's "build tool", we use this all the time. This is what compiles everything for us (by calling `rustc`).

Once you install these tools through the instructions they give you, you want to go into the `modkit/` code directory and run `cargo run`. That should both compile and run the code for you.

The advantage of compiling on your own is that when new changes are pushed, you can pull them and immediately use them. `cargo run` will recompile (if necessary) and run for you.

# Next Steps
Once you have the webserver running, you can start talking to it through websockets. See the [[Event API]] docs for more information.
