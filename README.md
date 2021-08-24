# SmartCast Api
[![crates.io](https://img.shields.io/crates/v/smartcast)](https://crates.io/crates/smartcast)
[![Documentation](https://docs.rs/smartcast/badge.svg)](https://docs.rs/smartcast)
[![Build](https://github.com/ConnorTroy/smartcast/actions/workflows/build.yml/badge.svg)](https://github.com/ConnorTroy/smartcast/actions/workflows/build.yml)

This library provides an API for connecting to and controlling Vizio SmartCast TVs and Speakers. The struct `Device` provides a client for interfacing with the SmartCast device.

## Get Started

You can use `smartcast::discover_devices()` to find SmartCast devices on your local network by issuing an [SSDP Query](https://en.wikipedia.org/wiki/Simple_Service_Discovery_Protocol) or attempt to connect directly using `Device::by_ip()` or `Device::by_uuid()`.

### Example

```rust
use smartcast::Device;

async fn example_main() -> Result<(), smartcast::Error> {

    let ssdp_devices = smartcast::discover_devices().await?;

    let dev_by_ssdp = ssdp_devices[0].clone();
    let ip_addr = dev_by_ssdp.ip();
    let uuid = dev_by_ssdp.uuid();

    let dev_by_ip = Device::from_ip(ip_addr).await?;
    let dev_by_uuid = Device::from_uuid(uuid).await?;

    assert_eq!(dev_by_ssdp.name(), dev_by_ip.name());
    assert_eq!(dev_by_ssdp.name(), dev_by_uuid.name());

    Ok(())
}
```

## Task List

- [x] Connect
- [x] Pairing
- [x] Readable settings
- [x] Get device state
- [x] Virtual remote commands
- [x] Writeable settings
- [ ] App launching
