mod support;
use support::{CodeSet, DeviceType, PortOption, Test};

#[tokio::test]
async fn dev_type_tv() {
    Test::simulate(
        PortOption::Random,
        DeviceType::TV,
        CodeSet::Random,
        |dev| async move {
            dev.settings().await.unwrap();
        },
    )
    .await;
}

#[tokio::test]
async fn dev_type_soundbar() {
    Test::simulate(
        PortOption::Random,
        DeviceType::SoundBar,
        CodeSet::Random,
        |dev| async move {
            dev.settings().await.unwrap();
        },
    )
    .await;
}
