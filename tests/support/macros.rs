#[macro_use]
macro_rules! device_desc {
    ($name:expr, $model:expr, $uuid:expr) => {
        format!(indoc::indoc! {
            "<?xml version=\"1.0\"?>
            <root xmlns=\"urn:schemas-upnp-org:device-1-0\">
                <specVersion>
                    <major>1</major>
                    <minor>0</minor>
                </specVersion>
                <URLBase>http://127.0.0.1:8008</URLBase>
                <device>
                    <deviceType>urn:dial-multiscreen-org:device:dial:1</deviceType>
                    <friendlyName>{}</friendlyName>
                    <manufacturer>Vizio</manufacturer>
                    <modelName>{}</modelName>
                    <UDN>uuid:{}</UDN>
                </device>
            </root>"
        },
        $name, $model, $uuid
        )
    };
}

#[macro_use]
macro_rules! status {
    ($result:expr) => {
        format!(
            r#""STATUS": {{
                "RESULT": "{}",
                "DETAIL": "{}"
            }}"#,
            $result.to_string().to_uppercase(),
            $result.to_string().to_lowercase()
        )
    };
}
