use smartcast::Device;

/// These tests will only work if there is a SmartCast device on the local network. Otherwise they will just pass.
#[tokio::test]
async fn find_by_ip() {
    if let Ok(devices) = smartcast::discover_devices().await {
        for dev in devices {
            if let Ok(same_dev) = Device::from_ip(dev.ip()).await {
                assert_eq!(dev.name(), same_dev.name());
                assert_eq!(dev.ip(), same_dev.ip());
                assert_eq!(dev.uuid(), same_dev.uuid());
            } else {
                panic!("Device could not be found by IP!");
            }
        }
    }
}

#[tokio::test]
async fn find_by_uuid() {
    if let Ok(devices) = smartcast::discover_devices().await {
        for dev in devices {
            if let Ok(same_dev) = Device::from_uuid(dev.uuid()).await {
                assert_eq!(dev.name(), same_dev.name());
                assert_eq!(dev.ip(), same_dev.ip());
                assert_eq!(dev.uuid(), same_dev.uuid());
            } else {
                panic!("Device could not be found by UUID!");
            }
        }
    }
}
