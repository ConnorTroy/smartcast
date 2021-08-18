# SmartCast Api

This library provides an API for connecting to and controlling Vizio SmartCast TVs and Speakers. The struct `Device` provides a client for interfacing with the SmartCast device.

## Get Started

You can use `discover_devices()` to find SmartCast devices on your local network by issuing an [SSDP Query](https://en.wikipedia.org/wiki/Simple_Service_Discovery_Protocol) or attempt to connect directly using `Device::by_ip()` or `Device::by_uuid()`.

### Example

```rust
use smartcast::Device;

async fn example_main() -> Result<(), smartcast::Error> {
    let ssdp_devices = smartcast::discover_devices().await?;

    let dev_by_ssdp = ssdp_devices[0].clone();
    let dev_by_ip = Device::from_ip(dev_by_ssdp.ip()).await?;
    let dev_by_uuid = Device::from_uuid(dev_by_ssdp.uuid()).await?;

    asserteq!(dev_by_ssdp.name(), dev_by_ip.name());
    asserteq!(dev_by_ssdp.name(), dev_by_uuid.name());
    Ok(())
}
```

## Task List

- [x] Connect
- [x] Pairing
- [x] Readable settings
- [x] Get device state
- [x] Virtual remote commands
- [ ] Writeable settings
- [ ] App launching
