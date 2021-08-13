mod support;
use support::{connect_device, simulate, DeviceType, PortOption};

#[tokio::test]
async fn dev_type_tv() {
    simulate(PortOption::Random, DeviceType::TV).await;
    let dev = connect_device().await;

    dev.settings().await.unwrap();
}

#[tokio::test]
async fn dev_type_soundbar() {
    simulate(PortOption::Random, DeviceType::SoundBar).await;
    let dev = connect_device().await;

    dev.settings().await.unwrap();
}
