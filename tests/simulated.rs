mod support;
use support::{helpers, CodeSet, DeviceType, PortOption, Test};

use smartcast::SettingType;

use rand::Rng;

#[tokio::test]
async fn pair_start() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Random,
        |dev| async move {
            dev.begin_pair("client_name", "client_id").await.unwrap();
        },
    )
    .await;
}

#[tokio::test]
async fn pair_finish() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Random,
        |mut dev| async move {
            let client_name = "name";
            let client_id = "id";

            let pairing_data = dev.begin_pair(client_name, client_id).await.unwrap();
            dev.finish_pair(pairing_data, "0000").await.unwrap();
        },
    )
    .await;
}

#[tokio::test]
async fn pair_cancel() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Random,
        |dev| async move {
            let client_name = "name";
            let client_id = "id";

            let pairing_data = dev.begin_pair(client_name, client_id).await.unwrap();
            dev.cancel_pair(pairing_data).await.unwrap();
        },
    )
    .await;
}

#[tokio::test]
async fn powerstate() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Random,
        |dev| async move {
            dev.is_powered_on().await.unwrap();
        },
    )
    .await;
}

#[tokio::test]
async fn current_input() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Random,
        |dev| async move {
            dev.current_input().await.unwrap();
        },
    )
    .await;
}

#[tokio::test]
async fn list_inputs() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Random,
        |dev| async move {
            dev.list_inputs().await.unwrap();
        },
    )
    .await;
}

#[tokio::test]
async fn change_input() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Random,
        |dev| async move {
            let inputs = dev.list_inputs().await.unwrap();
            for input in inputs {
                dev.change_input(input.name()).await.unwrap();
            }
            assert!(dev.change_input("not_an_input").await.is_err())
        },
    )
    .await;
}

#[tokio::test]
async fn settings_read() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Random,
        |dev| async move {
            let settings = dev.settings().await.unwrap();
            assert!(!settings.is_empty());
            for s in settings {
                match s.setting_type() {
                    SettingType::Slider => {
                        let found_slider_info = s.slider_info().await;
                        assert!(found_slider_info.is_ok());
                        assert!(&found_slider_info.unwrap().is_some());

                        let found_slider_info = s.slider_info().await.unwrap().unwrap();
                        let exp_slider = support::expected_slider_info();

                        if !found_slider_info.dec_marker.is_empty()
                            || !found_slider_info.inc_marker.is_empty()
                            || found_slider_info.center.is_some()
                        {
                            assert_eq!(exp_slider.dec_marker, found_slider_info.dec_marker);
                            assert_eq!(exp_slider.inc_marker, found_slider_info.inc_marker);
                            assert_eq!(exp_slider.center, found_slider_info.center);
                        }
                        assert_eq!(exp_slider.increment, found_slider_info.increment);
                        assert_eq!(exp_slider.max, found_slider_info.max);
                        assert_eq!(exp_slider.min, found_slider_info.min);
                    }
                    SettingType::Value => {
                        assert!(s.value::<serde_json::Value>().is_some())
                    }
                    SettingType::List | SettingType::XList => {
                        let elements = s.elements().await.unwrap();
                        assert!(elements.len() == support::LIST_LEN);
                    }
                    _ => {}
                }
            }
        },
    )
    .await;
}

#[tokio::test]
async fn settings_write() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Random,
        |dev| async move {
            let settings = dev.settings().await.unwrap();
            assert!(!settings.is_empty());

            let mut rng = rand::thread_rng();

            for s in settings {
                match s.setting_type() {
                    SettingType::Slider => {
                        log::debug!(target: "test::simulated", "Type Slider");
                        let slider_info = s.slider_info().await.unwrap().unwrap();

                        // Good Values
                        for _ in 0..50 {
                            assert!(s
                                .update(rng.gen_range(slider_info.min..=slider_info.max))
                                .await
                                .is_ok());
                        }
                        assert!(s.update(slider_info.min).await.is_ok());
                        assert!(s.update(slider_info.max).await.is_ok());

                        // Bad values - these should be handled by the library
                        assert!(s.update(slider_info.max + 1).await.is_err());
                        assert!(s.update(slider_info.max + 100).await.is_err());
                        assert!(s.update(slider_info.min - 1).await.is_err());
                        assert!(s.update(slider_info.min - 100).await.is_err());
                        assert!(s.update("bad value".to_string()).await.is_err());
                        assert!(s.update(true).await.is_err());
                    }
                    SettingType::Value => {
                        log::debug!(target: "test::simulated", "Type Value");
                        // Good Values
                        if s.is_boolean() {
                            log::debug!("Type Value - Boolean");
                            assert!(s.update(true).await.is_ok());
                            assert!(s.update(false).await.is_ok());
                        } else if s.is_number() {
                            log::debug!(target: "test::simulated", "Type Value - Number");
                            for _ in 0..50 {
                                assert!(s.update(rng.gen::<i32>()).await.is_ok())
                            }
                        }
                        if s.is_string() {
                            log::debug!(target: "test::simulated", "Type Value - String");
                            for _ in 0..50 {
                                assert!(s
                                    .update(support::rand_data::string(rng.gen_range(5..25)))
                                    .await
                                    .is_ok())
                            }
                        }

                        // Bad values - these should be handled by the library
                        if s.is_boolean() {
                            assert!(s.update(rng.gen::<f32>()).await.is_err());
                            assert!(s.update(rng.gen::<u32>()).await.is_err());
                            assert!(s.update(rng.gen::<i32>()).await.is_err());
                            assert!(s
                                .update(support::rand_data::string(rng.gen_range(5..25)))
                                .await
                                .is_err());
                        } else if s.is_number() {
                            for _ in 0..50 {
                                assert!(s
                                    .update(rng.gen_range(i32::MAX as f64..f64::MAX))
                                    .await
                                    .is_err());
                                assert!(s
                                    .update(rng.gen_range(i32::MAX as u64..u64::MAX))
                                    .await
                                    .is_err());
                                assert!(s
                                    .update(rng.gen_range(i32::MIN as i64..i64::MIN))
                                    .await
                                    .is_err());
                                assert!(s.update(true).await.is_err());
                                assert!(s.update(false).await.is_err());
                                assert!(s
                                    .update(support::rand_data::string(rng.gen_range(5..25)))
                                    .await
                                    .is_err());
                            }
                        } else if s.is_string() {
                            for _ in 0..50 {
                                assert!(s.update(rng.gen::<f32>()).await.is_err());
                                assert!(s.update(rng.gen::<u32>()).await.is_err());
                                assert!(s.update(rng.gen::<i32>()).await.is_err());
                                assert!(s.update(true).await.is_err());
                                assert!(s.update(false).await.is_err());
                                assert!(s
                                    .update(support::rand_data::string(rng.gen_range(100..250)))
                                    .await
                                    .is_err());
                            }
                        }
                    }
                    SettingType::List | SettingType::XList => {
                        log::debug!(target: "test::simulated", "Type List");
                        let elements = s.elements().await.unwrap();
                        assert!(elements.len() == support::LIST_LEN);

                        // Good Values
                        for element in elements {
                            assert!(s.update(element).await.is_ok());
                        }

                        // Bad values - these should be handled by the library
                        for _ in 0..50 {
                            assert!(s
                                .update(support::rand_data::string(rng.gen_range(10..25)))
                                .await
                                .is_err());
                        }
                    }
                    _ => {
                        // Bad values - these should be handled by the library
                        for _ in 0..50 {
                            assert!(s.update(rng.gen::<f32>()).await.is_err());
                            assert!(s.update(rng.gen::<u32>()).await.is_err());
                            assert!(s.update(rng.gen::<i32>()).await.is_err());
                            assert!(s.update(true).await.is_err());
                            assert!(s.update(false).await.is_err());
                            assert!(s
                                .update(support::rand_data::string(rng.gen_range(100..250)))
                                .await
                                .is_err());
                        }
                    }
                }
            }
        },
    )
    .await;
}

#[tokio::test]
async fn virtual_remote_default() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Default,
        |dev| async move {
            let buttons = helpers::button_vec();

            for i in 0..3 {
                for button in &buttons {
                    let res = match i {
                        0 => dev.key_down(*button, None).await,
                        1 => dev.key_up(*button).await,
                        2 => dev.key_press(*button).await,
                        _ => panic!(),
                    };
                    assert!(res.is_ok());
                }
            }
        },
    )
    .await;
}

#[tokio::test]
async fn virtual_remote_secondary() {
    Test::simulate(
        PortOption::Random,
        DeviceType::Random,
        CodeSet::Secondary,
        |dev| async move {
            let buttons = helpers::button_vec();

            for i in 0..3 {
                for button in &buttons {
                    let res = match i {
                        0 => dev.key_down(*button, None).await,
                        1 => dev.key_up(*button).await,
                        2 => dev.key_press(*button).await,
                        _ => panic!(),
                    };
                    assert!(res.is_ok());
                }
            }
        },
    )
    .await;
}
