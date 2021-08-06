use super::{Device, Result};

use regex::Regex;
use serde_json::Value;
use tokio::{
    net::UdpSocket,
    time::{timeout, Duration},
};

use std::net::SocketAddr;
use std::str;

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
    let socket = UdpSocket::bind({
        // "Connect" to a local ip to get local address
        let temp_socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0))).await?;
        temp_socket
            .connect(SocketAddr::from(([192, 168, 0, 1], 1)))
            .await?;
        temp_socket.local_addr()?
    })
    .await?;

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
    use super::ssdp;
    use crate::{constant::*, Device};

    use chrono::prelude::*;
    use indoc::indoc;
    use rand::{distributions::Alphanumeric, Rng};
    use tokio::{
        net::{TcpListener, UdpSocket},
        sync::oneshot::{self, Receiver as OneShotReceiver, Sender as OneShotSender},
        sync::watch::{self, Receiver, Sender},
    };

    use std::{io, net::SocketAddr};

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
    async fn emulate_ssdp(address_tx: OneShotSender<SocketAddr>, tx: Sender<Option<SocketAddr>>) {
        // Bind Socket
        let ssdp_socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .unwrap();

        // Communicate emulated ssdp address
        address_tx.send(ssdp_socket.local_addr().unwrap()).unwrap();

        let mut rbuf = [0; 1024];
        while let Ok((len, recv_addr)) = ssdp_socket.recv_from(&mut rbuf).await {
            // Send API address to emulated device for ssdp
            tx.send(Some(recv_addr)).unwrap();

            // Clear rbuf
            for b in rbuf[..len].iter_mut() {
                *b = 0
            }
        }
    }

    // Emulate Device SSDP Response
    async fn emulate_device(
        device_tx: Option<OneShotSender<Device>>,
        mut rx: Receiver<Option<SocketAddr>>,
    ) {
        // Bind Socket
        let device_socket = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .unwrap();
        let device_addr = device_socket.local_addr().unwrap();

        let rand_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .map(char::from)
            .take(32)
            .collect();

        // Build Fake Device
        let device = Device::new(
            format!("Fake Device-{}", &rand_string[0..4]), // name
            match device_tx {
                // manufacturer
                Some(_) => "Vizio",
                None => "Fake Company",
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

        // Return Device to be expected by main test
        if let Some(device_tx) = device_tx {
            device_tx.send(device.clone()).unwrap();
        }

        // Device Desc Server
        let desc_server = TcpListener::bind(SocketAddr::from((device_addr.ip(), 0)))
            .await
            .unwrap();
        let desc_addr = desc_server.local_addr().unwrap();
        let desc_endpoint: String = "ssdp/device-desc.xml".into();

        tokio::spawn({
            let device = device.clone();
            let device_ip = device_addr.ip();
            async move {
                let device_desc = device_desc!(
                    device.ip(),
                    device.port(),
                    device.name(),
                    device.manufacturer(),
                    device.model_name(),
                    device.uuid()
                );
                let body: &str = &[
                    "HTTP/1.1 200 OK",
                    &format!("Application-URL:http//{}:{}/apps/", device_ip, 0),
                    &format!("Content-Length:{}", device_desc.len()),
                    "Content-Type:application/xml",
                    "MAN: \"ssdp:discover\"",
                    "",
                    &device_desc,
                ]
                .join("\r\n");

                while let Ok((stream, _)) = desc_server.accept().await {
                    loop {
                        stream.writable().await.unwrap();
                        match stream.try_write(body.as_bytes()) {
                            Ok(_) => break,
                            Err(e) => {
                                if e.kind() == io::ErrorKind::WouldBlock {
                                    continue;
                                } else {
                                    panic!("{}", e);
                                }
                            }
                        }
                    }
                }
            }
        });

        // SSDP Response
        while rx.changed().await.is_ok() {
            let msg = *rx.borrow();
            if let Some(ip) = msg {
                let body = &[
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
                device_socket.send_to(body.as_bytes(), ip).await.unwrap();
            }
        }
    }

    #[tokio::test]
    async fn ssdp_single_device() {
        let (address_tx, address_rx) = oneshot::channel::<SocketAddr>();
        let (device_tx, device_rx) = oneshot::channel::<Device>();
        let (tx, rx) = watch::channel::<Option<SocketAddr>>(None);

        tokio::spawn(emulate_ssdp(address_tx, tx));
        tokio::spawn(emulate_device(Some(device_tx), rx.clone()));
        tokio::spawn(emulate_device(None, rx.clone()));

        let ssdp_addr: SocketAddr = address_rx.await.unwrap();
        let expected_device: Device = device_rx.await.unwrap();

        let devices = ssdp(
            &format!("{}:{}", ssdp_addr.ip(), ssdp_addr.port()),
            SSDP_URN,
            DEFAULT_SSDP_MAXTIME,
        )
        .await
        .unwrap();

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0], expected_device);
    }

    #[tokio::test]
    async fn ssdp_multi_device() {
        // Start SSDP
        let (address_tx, address_rx) = oneshot::channel::<SocketAddr>();
        let (tx, rx) = watch::channel::<Option<SocketAddr>>(None);
        tokio::spawn(emulate_ssdp(address_tx, tx));

        // Devices
        let mut device_rx: Vec<OneShotReceiver<Device>> = Vec::new();
        for _ in 0..10 {
            let (tx, dev_rx) = oneshot::channel::<Device>();
            device_rx.push(dev_rx);
            tokio::spawn(emulate_device(Some(tx), rx.clone()));
        }

        tokio::spawn(emulate_device(None, rx.clone()));

        let ssdp_addr: SocketAddr = address_rx.await.unwrap();
        let mut expected_devices: Vec<Device> = Vec::new();
        for rx in device_rx {
            expected_devices.push(rx.await.unwrap());
        }

        let mut devices = ssdp(
            &format!("{}:{}", ssdp_addr.ip(), ssdp_addr.port()),
            SSDP_URN,
            DEFAULT_SSDP_MAXTIME,
        )
        .await
        .unwrap();

        assert_eq!(devices.len(), 10);

        devices.sort_by(|a, b| a.name().partial_cmp(&b.name()).unwrap());
        expected_devices.sort_by(|a, b| a.name().partial_cmp(&b.name()).unwrap());
        assert_eq!(devices, expected_devices);
    }

    #[tokio::test]
    async fn ssdp_no_device() {
        let (address_tx, address_rx) = oneshot::channel::<SocketAddr>();
        let (tx, rx) = watch::channel::<Option<SocketAddr>>(None);

        tokio::spawn(emulate_ssdp(address_tx, tx));
        tokio::spawn(emulate_device(None, rx.clone()));
        tokio::spawn(emulate_device(None, rx.clone()));
        tokio::spawn(emulate_device(None, rx.clone()));
        tokio::spawn(emulate_device(None, rx.clone()));

        let ssdp_addr: SocketAddr = address_rx.await.unwrap();

        let devices = ssdp(
            &format!("{}:{}", ssdp_addr.ip(), ssdp_addr.port()),
            SSDP_URN,
            DEFAULT_SSDP_MAXTIME,
        )
        .await
        .unwrap();

        assert_eq!(devices.len(), 0);
    }
}
