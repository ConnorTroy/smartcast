mod support;
use support::{connect_device, simulate, CodeSet, DeviceType, PortOption};

#[tokio::test]
async fn dev_type_tv() {
    simulate(PortOption::Random, DeviceType::TV, CodeSet::Random).await;
    let dev = connect_device().await;

    dev.settings().await.unwrap();
}

#[tokio::test]
async fn dev_type_soundbar() {
    simulate(PortOption::Random, DeviceType::SoundBar, CodeSet::Random).await;
    let dev = connect_device().await;

    dev.settings().await.unwrap();
}
