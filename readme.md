# Modkit
This is the code than runs in the mailbox and exposes a WebSocket HTTP server.

## Running
This is a Rust package. If you need to install Rust and it's tooling, see here: https://rustup.rs/

Otherwise, you can run this crate with

```
$ cargo run --example modkit
```

This runs the `modkit` example, which is the main program that we use. There are other "example" programs in the `examples/` directory, mostly just used for testing the hardware.

Once the program is running, you should be able to run the front end on the same system and it will automatically connect.

## Environment Variables
This crates depends on a few environment variables to be set to run properly. They are as follows:

* `MODKIT_IMG_DIR` [default `~/modkit_images`]
    * The directory to place videos and images captured by the camera
* `MODKIT_FLIP_VERTICAL` [default `0`]
    * Set to `1` or `0` to flip the image/video vertically. We ended up mounting the camera upside down.
* `MODKIT_PIN` [default `6245`]
    * The login pin. The default spells `MAIL`
* `RUST_LOG`
    * The logging level to output when running. If not set, no output will be displayed.
    * I would set to `RUST_LOG=info`
* `DATABASE_URL`
    * The location of the database. I would supply an absolute path to the `sqlite` database like this:
    * `DATABASE_URL=sqlite:/home/me/foo/bar/modkit.db`


