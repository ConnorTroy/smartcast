#[cfg(not(test))]
pub const PORT_OPTIONS: [u16; 2] = [7345, 9000];

pub const DEFAULT_TIMEOUT: u64 = 3;
pub const SSDP_IP: &str = "239.255.255.250:1900";
pub const SSDP_URN: &str = "urn:dial-multiscreen-org:device:dial:1";

pub const DEFAULT_SSDP_MAXTIME: usize = 3;

// pub const APP_PAYLOAD_URL: &str =
//     "http://hometest.buddytv.netdna-cdn.com/appservice/app_availability_prod.json";
// pub const APP_NAME_URL: &str =
//     "http://hometest.buddytv.netdna-cdn.com/appservice/vizio_apps_prod.json";
