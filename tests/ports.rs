mod support;
use support::{connect_device, simulate, CodeSet, DeviceType, PortOption};

#[tokio::test]
async fn port7345() {
    simulate(PortOption::Port7345, DeviceType::Random, CodeSet::Random).await;
    connect_device().await;
}

#[tokio::test]
async fn port9000() {
    simulate(PortOption::Port9000, DeviceType::Random, CodeSet::Random).await;
    connect_device().await;
}
