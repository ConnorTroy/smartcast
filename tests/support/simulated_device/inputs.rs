use rand::Rng;

use std::collections::HashMap;

/// Struct to simulate an input on a device
#[derive(Debug)]
pub struct Input {
    pub cname: String,
    pub hashval: u32,
    pub name: String,
    pub friendly: String,
    pub readonly: bool,
}

/// Generates a list of inputs for the device
pub fn generate() -> HashMap<String, Input> {
    let mut rng = rand::thread_rng();
    let mut hash = HashMap::new();

    // SMARTCAST Input
    hash.insert(
        "CAST".into(),
        Input {
            cname: "cast".into(),
            hashval: rng.gen(),
            name: "CAST".into(),
            friendly: "SMARTCAST".into(),
            readonly: true,
        },
    );

    // HDMI
    for i in 0..4 {
        hash.insert(
            format!("HDMI-{}", i),
            Input {
                cname: format!("hdmi{}", i),
                hashval: rng.gen(),
                name: format!("HDMI-{}", i),
                friendly: format!("Device {}", rng.gen::<u16>()),
                readonly: false,
            },
        );
    }

    // COMPOSITE
    hash.insert(
        "COMP".into(),
        Input {
            cname: "comp".into(),
            hashval: rng.gen(),
            name: "COMP".into(),
            friendly: "COMP".into(),
            readonly: false,
        },
    );

    // TUNER
    hash.insert(
        "TV".into(),
        Input {
            cname: "tuner".into(),
            hashval: rng.gen(),
            name: "TV".into(),
            friendly: "TV".into(),
            readonly: false,
        },
    );
    hash
}
