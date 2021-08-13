mod support;
use support::{connect_device, simulate, DeviceType, PortOption};

use smartcast::SettingType;

use rand::Rng;

#[tokio::test]
async fn pair_start() {
    simulate(PortOption::Random, DeviceType::Random).await;
    let dev = connect_device().await;

    dev.begin_pair("client_name", "client_id").await.unwrap();
}

#[tokio::test]
async fn pair_finish() {
    simulate(PortOption::Random, DeviceType::Random).await;
    let mut dev = connect_device().await;
    let client_name = "name";
    let client_id = "id";

    let pairing_data = dev.begin_pair(client_name, client_id).await.unwrap();
    dev.finish_pair(pairing_data, "0000").await.unwrap();
}

#[tokio::test]
async fn pair_cancel() {
    simulate(PortOption::Random, DeviceType::Random).await;
    let dev = connect_device().await;
    let client_name = "name";
    let client_id = "id";

    let pairing_data = dev.begin_pair(client_name, client_id).await.unwrap();
    dev.cancel_pair(pairing_data).await.unwrap();
}

#[tokio::test]
async fn powerstate() {
    simulate(PortOption::Random, DeviceType::Random).await;
    let dev = connect_device().await;
    dev.is_powered_on().await.unwrap();
}

#[tokio::test]
async fn current_input() {
    simulate(PortOption::Random, DeviceType::Random).await;
    let dev = connect_device().await;

    dev.current_input().await.unwrap();
}

#[tokio::test]
async fn list_inputs() {
    simulate(PortOption::Random, DeviceType::Random).await;
    let dev = connect_device().await;

    dev.list_inputs().await.unwrap();
}

#[tokio::test]
async fn change_input() {
    simulate(PortOption::Random, DeviceType::Random).await;
    let dev = connect_device().await;

    let inputs = dev.list_inputs().await.unwrap();
    for input in inputs {
        dev.change_input(input.name()).await.unwrap();
    }
    // TODO: Bad input
}

#[tokio::test]
async fn read_settings() {
    simulate(PortOption::Random, DeviceType::Random).await;
    let dev = connect_device().await;

    let settings = dev.settings().await.unwrap();
    assert!(!settings.is_empty());
    for s in settings {
        match s.setting_type() {
            SettingType::Slider => {
                assert_eq!(
                    support::expected_slider_info(),
                    s.slider_info().await.unwrap().unwrap()
                )
            },
            SettingType::Value => {
                assert!(s.value::<serde_json::Value>().is_some())
            },
            SettingType::List | SettingType::XList => {
                let elements = s.elements().await.unwrap().unwrap();
                assert!(elements.len() == support::LIST_LEN);
            }
            _ => {}
        }
    }
}

#[tokio::test]
async fn write_settings() {
    simulate(PortOption::Random, DeviceType::Random).await;
    let dev = connect_device().await;

    let settings = dev.settings().await.unwrap();
    assert!(!settings.is_empty());

    let mut rng = rand::thread_rng();

    for s in settings {
        match s.setting_type() {
            SettingType::Slider => {
                let slider_info = s.slider_info().await.unwrap().unwrap();

                // Good Values
                for _ in 0..50 {
                    assert!(s.write(rng.gen_range(slider_info.min..=slider_info.max)).await.is_ok());
                }
                assert!(s.write(slider_info.min).await.is_ok());
                assert!(s.write(slider_info.max).await.is_ok());

                // Bad values - these should be handled by the library
                assert!(s.write(slider_info.max + 1).await.is_err());
                assert!(s.write(slider_info.max + 100).await.is_err());
                assert!(s.write(slider_info.min - 1).await.is_err());
                assert!(s.write(slider_info.min - 100).await.is_err());
                assert!(s.write("bad value").await.is_err());
                assert!(s.write(true).await.is_err());
            },
            SettingType::Value => {
                // Good Values
                if s.is_boolean() {
                    assert!(s.write(true).await.is_ok());
                    assert!(s.write(false).await.is_ok());
                }
                else if s.is_number(){
                    for _ in 0..50 {
                        assert!(s.write(rng.gen::<i32>()).await.is_ok())
                    }
                }
                if s.is_string() {
                    for _ in 0..50 {
                        assert!(s.write(support::rand_data::string(rng.gen_range(5..25))).await.is_ok())
                    }
                }

                // Bad values - these should be handled by the library
                if s.is_boolean() {
                    assert!(s.write(rng.gen::<f32>()).await.is_err());
                    assert!(s.write(rng.gen::<u32>()).await.is_err());
                    assert!(s.write(rng.gen::<i32>()).await.is_err());
                    assert!(s.write(support::rand_data::string(rng.gen_range(5..25))).await.is_err());
                }
                else if s.is_number() {
                    for _ in 0..50 {
                        assert!(s.write(rng.gen::<f64>()).await.is_err());
                        assert!(s.write(rng.gen_range(i32::MAX as u64..u64::MAX)).await.is_err());
                        assert!(s.write(rng.gen_range(i32::MIN as i64..i64::MIN)).await.is_err());
                        assert!(s.write(true).await.is_err());
                        assert!(s.write(false).await.is_err());
                        assert!(s.write(support::rand_data::string(rng.gen_range(5..25))).await.is_err());
                    }
                }
                else if s.is_string() {
                    for _ in 0..50 {
                        assert!(s.write(rng.gen::<f32>()).await.is_err());
                        assert!(s.write(rng.gen::<u32>()).await.is_err());
                        assert!(s.write(rng.gen::<i32>()).await.is_err());
                        assert!(s.write(true).await.is_err());
                        assert!(s.write(false).await.is_err());
                        assert!(s.write(support::rand_data::string(rng.gen_range(100..250))).await.is_err());
                    }
                }
            },
            SettingType::List | SettingType::XList => {
                let elements = s.elements().await.unwrap().unwrap();
                assert!(elements.len() == support::LIST_LEN);

                // Good Values
                for element in elements {
                    assert!(s.write(element).await.is_ok());
                }

                // Bad values - these should be handled by the library
                for _ in 0..50 {
                    assert!(s.write(support::rand_data::string(rng.gen_range(10..25))).await.is_err());
                }
            }
            _ => {
                // Bad values - these should be handled by the library
                for _ in 0..50 {
                    assert!(s.write(rng.gen::<f32>()).await.is_err());
                    assert!(s.write(rng.gen::<u32>()).await.is_err());
                    assert!(s.write(rng.gen::<i32>()).await.is_err());
                    assert!(s.write(true).await.is_err());
                    assert!(s.write(false).await.is_err());
                    assert!(s.write(support::rand_data::string(rng.gen_range(100..250))).await.is_err());
                }
            }
        }
    }
}
