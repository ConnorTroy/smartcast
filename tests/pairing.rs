mod support;
use support::{emulate, connect_device, PortOption};

#[tokio::test]
async fn pair_start() {
    emulate(PortOption::Port7345).await;
    let dev = connect_device().await;

    dev.begin_pair("client_name", "client_id").await.unwrap();
}

#[tokio::test]
async fn pair_finish() {
    emulate(PortOption::Port7345).await;
    let mut dev = connect_device().await;
    let client_name = "name";
    let client_id = "id";

    let (token, challenge) = dev.begin_pair(client_name, client_id).await.unwrap();
    dev.finish_pair(client_id, token, challenge, "0000").await.unwrap();
}

#[tokio::test]
async fn pair_cancel() {
    emulate(PortOption::Port7345).await;
    let dev = connect_device().await;
    let client_name = "name";
    let client_id = "id";

    dev.begin_pair(client_name, client_id).await.unwrap();
    dev.cancel_pair(client_name, client_id).await.unwrap();
}

// #[tokio::test]
// async fn pair_finish_8000() {
//     emulate(PortOption::Port8000).await;
//     let mut dev = connect_device().await;
//     let client_name = "name";
//     let client_id = "id";

//     let (token, challenge) = dev.begin_pair(client_name, client_id).await.unwrap();
//     dev.finish_pair(client_id, token, challenge, "0000").await.unwrap();
// }
