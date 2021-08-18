mod support;
use support::{CodeSet, DeviceType, PortOption, Test};

#[tokio::test]
async fn port7345() {
    Test::simulate(
        PortOption::Port7345,
        DeviceType::Random,
        CodeSet::Random,
        |_| async move {},
    )
    .await;
}

#[tokio::test]
async fn port9000() {
    Test::simulate(
        PortOption::Port9000,
        DeviceType::Random,
        CodeSet::Random,
        |_| async move {},
    )
    .await;
}
