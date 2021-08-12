mod support;
use support::{connect_device, emulate, DeviceType, PortOption};

#[tokio::test]
async fn port7345() {
    emulate(PortOption::Port7345, DeviceType::Random).await;
    connect_device().await;
}

#[tokio::test]
async fn port9000() {
    emulate(PortOption::Port9000, DeviceType::Random).await;
    connect_device().await;
}
