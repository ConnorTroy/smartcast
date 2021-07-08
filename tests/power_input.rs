mod support;
use support::{emulate, connect_device, PortOption};

#[tokio::test]
async fn powerstate() {
    emulate(PortOption::Port7345).await;
    let dev = connect_device().await;
    dev.is_powered_on().await.unwrap();
}

#[tokio::test]
async fn current_input() {
    emulate(PortOption::Port7345).await;
    let dev = connect_device().await;

    dev.current_input().await.unwrap();
}

#[tokio::test]
async fn list_inputs() {
    emulate(PortOption::Port7345).await;
    let dev = connect_device().await;

    dev.list_inputs().await.unwrap();
}

#[tokio::test]
async fn change_input() {
    emulate(PortOption::Port7345).await;
    let dev = connect_device().await;

    let inputs = dev.list_inputs().await.unwrap();
    println!("{:#?}", inputs);
    for input in inputs {
        dev.change_input(input.name()).await.unwrap();
    }
}

// #[tokio::test]
// async fn change_input_8000() {
//     emulate(PortOption::Port8000).await;
//     let dev = connect_device().await;

//     let inputs = dev.list_inputs().await.unwrap();
//     println!("{:#?}", inputs);
//     for input in inputs {
//         dev.change_input(input.name()).await.unwrap();
//     }
// }
