use std::{sync::Arc, time::Duration};

use rust_extensions::{ApplicationStates, Logger, MyTimer, MyTimerTick};
use serde_derive::Serialize;

pub struct MyTelemetryWriter {
    timer: MyTimer,
    telemetry_timer: Arc<TelemetryTimer>,
}

impl MyTelemetryWriter {
    pub fn new(url: String, app_name: String) -> Self {
        let mut result = Self {
            timer: MyTimer::new(Duration::from_secs(1)),
            telemetry_timer: Arc::new(TelemetryTimer { url, app_name }),
        };

        result
            .timer
            .register_timer("TelemetryWriterTimer", result.telemetry_timer.clone());
        result
    }

    pub fn start(
        &self,
        app_states: Arc<dyn ApplicationStates + Send + Sync + 'static>,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) {
        if my_telemetry::TELEMETRY_INTERFACE.is_telemetry_set_up() {
            return;
        }

        my_telemetry::TELEMETRY_INTERFACE
            .writer_is_set
            .store(true, std::sync::atomic::Ordering::SeqCst);
        self.timer.start(app_states, logger);
        println!("Telemetry writer is started");
    }
}

pub struct TelemetryTimer {
    url: String,
    app_name: String,
}

#[async_trait::async_trait]
impl MyTimerTick for TelemetryTimer {
    async fn tick(&self) {
        let to_write = {
            let mut write_access = my_telemetry::TELEMETRY_INTERFACE
                .telemetry_collector
                .lock()
                .await;

            write_access.get_events()
        };

        if to_write.is_none() {
            return;
        }

        let to_write = to_write.unwrap();

        let mut json_model = Vec::with_capacity(to_write.len());

        for itm in to_write {
            let json_item = TelemetryHttpModel {
                process_id: itm.process_id,
                started: itm.started,
                ended: itm.finished,
                service_name: self.app_name.clone(),
                event_data: itm.data,
                success: itm.success,
                fail: itm.fail,
                ip: itm.ip,
            };

            json_model.push(json_item);
        }

        let body = serde_json::to_vec(&json_model).unwrap();

        let flurl = flurl::FlUrl::new(self.url.as_str(), None)
            .append_path_segment("api")
            .append_path_segment("add")
            .post(Some(body))
            .await;

        if let Err(err) = flurl {
            println!("Can not write telemetry: {:?}", err);
        }
    }
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
