# mod_tile_rs
A Rust implementation of the tile module for Apache 2 webserver.

## Testing
Apache 2 does not have a library that modules can link to, rather they are linked into the apache2 process. The unit tests of modules similarly don't have an Apache 2 library to link against and normally linking will fail to due unresolved ap_ symbols. To work around this run the tests via:
> cargo rustc -- --test -C link-args=-Wl,--unresolved-symbols=ignore-in-object-files
> ./target/debug/deps/mod_tile_rs
