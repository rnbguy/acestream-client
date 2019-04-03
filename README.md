# Acestream client

This work is inspired by [jonian/acestream-launcher](https://github.com/jonian/acestream-launcher).

I intend to use this as a Rust library. But there is an example binary `aceplay.rs` to open streams on `mpv` player. You can install it using

    cargo install --example aceplay acestream_client
    // play Sintel - https://en.wikipedia.org/wiki/Sintel
    aceplay acestream://94c2fd8fb9bc8f2fc71a2cbe9d4b866f227a0209

It requires `acestreamengine` and `mpv` installed on your computer. Of course, `cargo` is needed to build the binary.
