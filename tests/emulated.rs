mod support;
use support::{connect_device, emulate, DeviceType, PortOption};

#[tokio::test]
async fn pair_start() {
    emulate(PortOption::Random, DeviceType::TV).await;
    let dev = connect_device().await;

    dev.begin_pair("client_name", "client_id").await.unwrap();
}

#[tokio::test]
async fn pair_finish() {
    emulate(PortOption::Random, DeviceType::TV).await;
    let mut dev = connect_device().await;
    let client_name = "name";
    let client_id = "id";

    let pairing_data = dev.begin_pair(client_name, client_id).await.unwrap();
    dev.finish_pair(pairing_data, "0000").await.unwrap();
}

#[tokio::test]
async fn pair_cancel() {
    emulate(PortOption::Random, DeviceType::TV).await;
    let dev = connect_device().await;
    let client_name = "name";
    let client_id = "id";

    let pairing_data = dev.begin_pair(client_name, client_id).await.unwrap();
    dev.cancel_pair(pairing_data).await.unwrap();
}

#[tokio::test]
async fn powerstate() {
    emulate(PortOption::Random, DeviceType::TV).await;
    let dev = connect_device().await;
    dev.is_powered_on().await.unwrap();
}

#[tokio::test]
async fn current_input() {
    emulate(PortOption::Random, DeviceType::TV).await;
    let dev = connect_device().await;

    dev.current_input().await.unwrap();
}

#[tokio::test]
async fn list_inputs() {
    emulate(PortOption::Random, DeviceType::TV).await;
    let dev = connect_device().await;

    dev.list_inputs().await.unwrap();
}

#[tokio::test]
async fn change_input() {
    emulate(PortOption::Random, DeviceType::TV).await;
    let dev = connect_device().await;

    let inputs = dev.list_inputs().await.unwrap();
    println!("{:#?}", inputs);
    for input in inputs {
        dev.change_input(input.name()).await.unwrap();
    }
}
