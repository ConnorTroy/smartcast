use super::{Device, Result};
use super::constants::DEFAULT_TIMEOUT;

use regex::Regex;

use std::net::UdpSocket;
use std::time::Duration;
use std::str;

async fn uaudp_followup(ssdp_response: &[u8; 1024]) -> Result<Option<Device>> {
    // Parse headers for xml url
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut res= httparse::Response::new(&mut headers);

    res.parse(ssdp_response).unwrap();

    let location =
        str::from_utf8(
            match
            headers.iter()
            .find(|x| x.name.to_lowercase() == "location") {
                Some(header) => header.value,
                None => return Ok(None)
            }
        ).unwrap();

    // Get device description xml
    let res =
        reqwest::get(location).await?
        .text().await?;

    // Parse xml for device info
    let re = Regex::new(r"<([^>]+)>([^<]+)</([^>]+)>").unwrap();

    let mut friendly_name: Option<String> = None;
    let mut manufacturer: Option<String> = None;
    let mut model_name: Option<String> = None;
    let mut ip_addr: Option<String> = None;
    let mut uuid: Option<String> = None;

    for cap in re.captures_iter(&res) {
        if cap.get(1).unwrap().as_str() == cap.get(3).unwrap().as_str() {
            let field = cap.get(2).unwrap().as_str();
            match cap.get(1).unwrap().as_str() {
                "URLBase" => {
                    // Extract ip from base url (strip http and port)
                    let mut field_chars = field.chars();
                    let ip_start = field_chars.position(|c| c == '/').unwrap() + 1;
                    let ip_end = field_chars.position(|c| c == ':').unwrap();
                    ip_addr = Some(
                        field_chars
                        .skip(ip_start)
                        .take(ip_end - ip_start)
                        .collect()
                    );
                }
                "friendlyName" => {
                    friendly_name = Some(field.to_string());
                },
                "manufacturer" => {
                    manufacturer = Some(field.to_string());
                },
                "modelName" => {
                    model_name = Some(field.to_string());
                },
                "UDN" => {
                    uuid = Some(field[5..].to_string());
                },
                _ => {},
            }
        }
    }

    match (friendly_name, manufacturer, model_name, ip_addr, uuid) {
        (
        Some(friendly_name),
        Some(manufacturer),
        Some(model_name),
        Some(ip_addr),
        Some(uuid)
        ) if manufacturer == "Vizio" => {
            Ok(Some(
                Device::new (
                    friendly_name,
                    manufacturer,
                    model_name,
                    ip_addr,
                    uuid,
                )
            ))
        },
        _ => Ok(None)
    }

}

// Returns a vector of Vizio Devices
pub(crate) async fn ssdp(host: &str, service: &str, max: u8) -> Result<Vec<Device>> {
    let body: &str =
        &[
            "M-SEARCH * HTTP/1.1",
            &format!("HOST: {}", host),
            "MAN: \"ssdp:discover\"",
            &format!("ST: {}", service),
            &format!("MX: {}", max),
            "",
            ""
        ].join("\r\n");

    // Open UDP Socket
    // TO-DO: get local ip
    // TO-DO: backup ports
    let socket = UdpSocket::bind("192.168.0.8:32000")?;
    socket.set_read_timeout(Some(Duration::from_secs(DEFAULT_TIMEOUT)))?;

    // Send ssdp request
    socket.send_to(body.as_bytes(), host)?;
    let mut rbuf = [0; 1024];

    // Get responses from devices
    let mut devices: Vec<Device> = Vec::new();
    while let Ok(_) = socket.recv(&mut rbuf) {
        match uaudp_followup(&rbuf).await? {
            Some(device) => devices.push(device),
            _ => {},
        }
    }

    Ok(devices)
}
