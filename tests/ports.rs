mod support;
use support::{connect_device, emulate, PortOption};

#[tokio::test]
async fn port7345() {
    emulate(PortOption::Port7345).await;
    connect_device().await;
}

#[tokio::test]
async fn port9000() {
    emulate(PortOption::Port9000).await;
    connect_device().await;
}
