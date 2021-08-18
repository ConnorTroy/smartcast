use super::{Device, Result};

use regex::Regex;
use serde_json::Value;
use tokio::{
    net::UdpSocket,
    time::{timeout, Duration},
};

use std::net::SocketAddr;
use std::str;

pub const SSDP_IP: &str = "239.255.255.250:1900";
pub const SSDP_URN: &str = "urn:dial-multiscreen-org:device:dial:1";
pub const DEFAULT_SSDP_MAXTIME: usize = 3;

pub(super) async fn uaudp_followup(location: &str) -> Result<Option<Device>> {
    // Get device description xml
    let res = reqwest::get(location).await?.text().await?;

    // Parse xml for device info
    let mut items: Value = serde_xml_rs::from_str(&res).unwrap();

    let friendly_name =
        serde_json::from_value::<String>(items["device"]["friendlyName"]["$value"].take());
    let manufacturer =
        serde_json::from_value::<String>(items["device"]["manufacturer"]["$value"].take());
    let model_name =
        serde_json::from_value::<String>(items["device"]["modelName"]["$value"].take());
    let uuid = serde_json::from_value::<String>(items["device"]["UDN"]["$value"].take());

    match (friendly_name, manufacturer, model_name, uuid) {
        (Ok(friendly_name), Ok(manufacturer), Ok(model_name), Ok(uuid))
            if manufacturer == "Vizio" =>
        {
            // Strip http and port
            let ip_addr = Regex::new(r"(?:http:////)?(\d+\.\d+\.\d+\.\d+)(?::\d+)?")
                .unwrap()
                .captures(location)
                .unwrap()[1]
                .into();
            // Strip uuid
            let uuid = Regex::new(r"^(?:(?:\s*\w+)\s*:\s*)?(.*)")
                .unwrap()
                .captures(&uuid)
                .unwrap()[1]
                .into();

            Ok(Some(
                Device::new(friendly_name, manufacturer, model_name, ip_addr, uuid).await?,
            ))
        }
        _ => Ok(None),
    }
}

// Returns a vector of Vizio Devices
pub(super) async fn ssdp(host: &str, st: &str, mx: usize) -> Result<Vec<Device>> {
    let body: &str = &[
        "M-SEARCH * HTTP/1.1",
        &format!("HOST: {}", host),
        "MAN: \"ssdp:discover\"",
        &format!("ST: {}", st),
        &format!("MX: {}", mx),
        "",
        "",
    ]
    .join("\r\n");

    // Open UDP Socket
    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0))).await?;

    // Send ssdp request
    socket.send_to(body.as_bytes(), host).await?;
    let mut rbuf = [0; 1024];

    // Get responses from devices
    let mut devices: Vec<Device> = Vec::new();
    while let Ok(Ok(len)) = timeout(Duration::from_secs(mx as u64), socket.recv(&mut rbuf)).await {
        // Parse headers for xml url
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut res = httparse::Response::new(&mut headers);

        res.parse(&rbuf).unwrap();

        let location = str::from_utf8(
            match headers.iter().find(|x| x.name.to_lowercase() == "location") {
                Some(header) => header.value,
                None => continue,
            },
        )
        .unwrap();

        if let Some(device) = uaudp_followup(location).await? {
            devices.push(device);
        }
        // Clear rbuf
        for b in rbuf[..len].iter_mut() {
            *b = 0
        }
    }

    Ok(devices)
}

#[cfg(test)]
mod tests {
    use super::{SSDP_URN, DEFAULT_SSDP_MAXTIME, ssdp};
    use crate::{Device};

    use chrono::prelude::*;
    use http::Response;
    use indoc::indoc;
    use rand::{distributions::Alphanumeric, Rng};
    use tokio::{
        net::UdpSocket,
        sync::watch::{self, Receiver},
    };
    use warp::{self, Filter};

    use std::net::SocketAddr;

    macro_rules! device_desc {
        ($ip:expr, $port:expr, $name:expr, $manufacturer:expr, $model:expr, $uuid:expr) => {
            format!(
                indoc! {
                "<?xml version=\"1.0\"?>\r
                <root xmlns=\"urn:schemas-upnp-org:device-1-0\">\r
                    <specVersion>\r
                        <major>1</major>\r
                        <minor>0</minor>\r
                    </specVersion>\r
                    <URLBase>http://{}:{}</URLBase>\r
                    <device>\r
                        <deviceType>urn:dial-multiscreen-org:device:dial:1</deviceType>\r
                        <friendlyName>{}</friendlyName>\r
                        <manufacturer>{}</manufacturer>\r
                        <modelName>{}</modelName>\r
                        <UDN>uuid:{}</UDN>\r
                    </device>\r
                </root>\r
                "
                },
                $ip, $port, $name, $manufacturer, $model, $uuid
            )
        };
    }

    // Emulate Network SSDP
    async fn emulate_ssdp() -> (SocketAddr, Receiver<Option<SocketAddr>>) {
        // Emulate ssdp with watch
        let (ssdp_tx, ssdp_rx) = watch::channel::<Option<SocketAddr>>(None);

        // Bind Socket
        let ssdp_socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .unwrap();

        let ssdp_addr = ssdp_socket.local_addr().unwrap();
        tokio::spawn(async move {
            let mut rbuf = [0; 1024];
            while let Ok((len, recv_addr)) = ssdp_socket.recv_from(&mut rbuf).await {
                // Send API address to emulated device for ssdp
                ssdp_tx.send(Some(recv_addr)).unwrap();

                // Clear rbuf
                for b in rbuf[..len].iter_mut() {
                    *b = 0
                }
            }
        });
        (ssdp_addr, ssdp_rx)
    }

    // Emulate Device SSDP Response
    async fn emulate_device(
        smartcast_device: bool,
        mut rx: Receiver<Option<SocketAddr>>,
    ) -> Device {
        // Bind Socket
        let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .unwrap();
        let device_addr = socket.local_addr().unwrap();

        // Device Desc Server
        let desc_addr: SocketAddr = {
            let socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
                .await
                .unwrap();
            socket.local_addr().unwrap()
        };

        // Build Device
        let rand_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(32)
            .collect();

        let device = Device::new(
            format!("Fake Device-{}", &rand_string[0..4]), // name
            match smartcast_device {
                // manufacturer
                true => "Vizio",
                false => "Fake Company",
            }
            .into(),
            format!("fake_model_{}", &rand_string[4..8]), // model
            device_addr // ip_addr
                .ip()
                .to_string(),
            format!(
                "{}-{}-{}-{}-{}", // uuid
                &rand_string[0..8],
                &rand_string[8..12],
                &rand_string[12..16],
                &rand_string[16..20],
                &rand_string[20..32]
            ),
        )
        .await
        .unwrap();
        let desc_endpoint: String = "ssdp/device-desc.xml".into();

        // --- Device Description Server
        let descriptions = warp::path("ssdp")
            .and(warp::path("device-desc.xml"))
            .and(warp::path::end())
            .and(warp::get())
            .map({
                let ip = device.ip();
                let port = device.port();
                let name = device.name();
                let manufacturer = device.manufacturer();
                let model_name = device.model_name();
                let uuid = device.uuid();
                move || {
                    let desc_xml = device_desc!(ip, port, name, manufacturer, model_name, uuid);
                    Response::builder()
                        .header("Application-URL", "http//127.0.0.1:8008/apps/")
                        .header("Content-Length", desc_xml.len())
                        .header("Content-Type", "application/xml")
                        .body(desc_xml)
                        .unwrap()
                }
            });

        tokio::spawn(warp::serve(descriptions).run(desc_addr));

        // SSDP Response
        tokio::spawn({
            let body = [
                "HTTP/1.1 200 OK",
                "CACHE-CONTROL: max-age=1800",
                &format!("DATE: {}", Utc::now().format("%a, %d %b %Y %X GMT")),
                "EXT:",
                &format!(
                    "LOCATION: http://{}:{}/{}",
                    desc_addr.ip(),
                    desc_addr.port(),
                    desc_endpoint
                ),
                "OPT: \"http://schemas.upnp.org/upnp/1/0/\"; ns=01",
                "SERVER: Linux/4.19.71+, UPnP/1.0, Portable SDK for UPnP devices/1.6.18",
                "X-User-Agent: redsonic",
                "ST: urn:dial-multiscreen-org:device:dial:1",
                &format!(
                    "USN: uuid:{}::urn:dial-multiscreen-org:device:dial:1",
                    device.uuid()
                ),
                "BOOTID.UPNP.ORG: 0",
                "CONFIGID.UPNP.ORG: 3",
                "",
                "",
            ]
            .join("\r\n");
            async move {
                while rx.changed().await.is_ok() {
                    let msg = *rx.borrow();
                    if let Some(ip) = msg {
                        socket.send_to(body.as_bytes(), ip).await.unwrap();
                    }
                }
            }
        });

        device
    }

    #[tokio::test]
    async fn ssdp_single_device() {
        // Start SSDP
        let (ssdp_addr, ssdp_rx) = emulate_ssdp().await;

        // Devices
        let expected_device = emulate_device(true, ssdp_rx.clone()).await;
        emulate_device(false, ssdp_rx.clone()).await;

        let found_devices = ssdp(
            &format!("{}:{}", ssdp_addr.ip(), ssdp_addr.port()),
            SSDP_URN,
            DEFAULT_SSDP_MAXTIME,
        )
        .await
        .unwrap();

        assert_eq!(found_devices.len(), 1);
        assert_eq!(found_devices[0], expected_device);
    }

    #[tokio::test]
    async fn ssdp_multi_device() {
        // Start SSDP
        let (ssdp_addr, ssdp_rx) = emulate_ssdp().await;

        // Devices
        let mut expected_devices: Vec<Device> = Vec::new();
        for _ in 0..10 {
            expected_devices.push(emulate_device(true, ssdp_rx.clone()).await);
        }
        emulate_device(false, ssdp_rx).await;

        let mut found_devices = ssdp(
            &format!("{}:{}", ssdp_addr.ip(), ssdp_addr.port()),
            SSDP_URN,
            DEFAULT_SSDP_MAXTIME,
        )
        .await
        .unwrap();

        assert_eq!(found_devices.len(), 10);

        found_devices.sort_by(|a, b| a.name().partial_cmp(&b.name()).unwrap());
        expected_devices.sort_by(|a, b| a.name().partial_cmp(&b.name()).unwrap());
        assert_eq!(found_devices, expected_devices);
    }

    #[tokio::test]
    async fn ssdp_no_device() {
        // Start SSDP
        let (ssdp_addr, ssdp_rx) = emulate_ssdp().await;

        // Devices
        for _ in 0..10 {
            emulate_device(false, ssdp_rx.clone()).await;
        }

        let found_devices = ssdp(
            &format!("{}:{}", ssdp_addr.ip(), ssdp_addr.port()),
            SSDP_URN,
            DEFAULT_SSDP_MAXTIME,
        )
        .await
        .unwrap();

        assert_eq!(found_devices.len(), 0);
    }
}
