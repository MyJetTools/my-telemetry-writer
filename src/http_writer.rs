use my_telemetry::TelemetryEvent;
use serde::*;

pub async fn write_as_http(url: &str, app_name: &str, to_write: Vec<TelemetryEvent>) -> bool {
    let mut json_model = Vec::with_capacity(to_write.len());

    for itm in to_write {
        let json_item = TelemetryHttpModel {
            process_id: itm.process_id,
            started: itm.started,
            ended: itm.finished,
            service_name: app_name.to_string(),
            event_data: itm.data,
            success: itm.success,
            fail: itm.fail,
            ip: itm.ip,
        };

        json_model.push(json_item);
    }

    let body = serde_json::to_vec(&json_model).unwrap();

    let flurl = flurl::FlUrl::new(url)
        .append_path_segment("api")
        .append_path_segment("add")
        .post(Some(body))
        .await;

    if let Err(err) = flurl {
        println!("Can not write telemetry: {:?}", err);
        return false;
    }

    true
}

#[derive(Serialize)]
pub struct TelemetryHttpModel {
    #[serde(rename = "processId")]
    pub process_id: i64,
    #[serde(rename = "started")]
    pub started: i64,
    #[serde(rename = "ended")]
    pub ended: i64,
    #[serde(rename = "serviceName")]
    pub service_name: String,
    #[serde(rename = "eventData")]
    pub event_data: String,
    pub success: Option<String>,
    pub fail: Option<String>,
    pub ip: Option<String>,
}
