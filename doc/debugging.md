# Debugging Kuksa Databroker

For debugging, troubleshooting and specific testing purposes, databroker can be built with a different build profile `release-with-debug`, which enables some symbols to remain in the binary.

```sh
# in ${WORKSPACE}
cargo build --profile release-with-debug --all-targets
```

 _Note:_ The size of the databroker (x86_64) binary increases from ~6 MiB to ~60 MiB for the debug build.

## Profile Configuration

See [The Cargo Book](https://doc.rust-lang.org/cargo/reference/profiles.html) for details on profile settings.

- Link Time Optimization (LTO), such as dead code removal, is enabled.
- Optimization Level is "s": optimize for binary size
- Codegen Units is set to 1 for faster code (increased compile time)
- Incremental is disabled for release profiles (no improvements between recompiles)
- Strip is disabled, to keep debug symbols (necessary for eBPF to work)
- Debug is enabled (full debug info)

```yaml
[profile.release]
lto = true
opt-level = "s"
codegen-units = 1
incremental = false
strip = true

[profile.release-with-debug]
inherits = "release"
strip = false
debug = true
```
