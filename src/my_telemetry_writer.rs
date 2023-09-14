use std::{sync::Arc, time::Duration};

use rust_extensions::{ApplicationStates, Logger, MyTimer, MyTimerTick, StrOrString};

use crate::{
    grpc_writer::GrpcClient,
    write_mode::{WriteMode, WriteModeKeeper},
    MyTelemetrySettings,
};

pub struct MyTelemetryWriter {
    timer: MyTimer,
    telemetry_timer: Arc<TelemetryTimer>,
}

impl MyTelemetryWriter {
    pub fn new(
        app_name: impl Into<StrOrString<'static>>,
        settings: Arc<dyn MyTelemetrySettings + Send + Sync + 'static>,
    ) -> Self {
        let app_name = app_name.into();
        let mut result = Self {
            timer: MyTimer::new(Duration::from_secs(1)),
            telemetry_timer: Arc::new(TelemetryTimer::new(settings, app_name)),
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
    settings: Arc<dyn MyTelemetrySettings + Send + Sync + 'static>,
    app_name: StrOrString<'static>,
    write_mode: Arc<WriteModeKeeper>,
    grpc_client: GrpcClient,
}

impl TelemetryTimer {
    pub fn new(
        settings: Arc<dyn MyTelemetrySettings + Send + Sync + 'static>,
        app_name: StrOrString<'static>,
    ) -> Self {
        Self {
            app_name,
            write_mode: Arc::new(WriteModeKeeper::new()),
            grpc_client: GrpcClient::new(settings.clone()),
            settings,
        }
    }

    async fn detect_write_mode(&self) {
        if self.grpc_client.is_grpc().await {
            self.write_mode.set_write_mode(WriteMode::Grpc);
            return;
        }

        self.write_mode.set_write_mode(WriteMode::Http);
    }
}

#[async_trait::async_trait]
impl MyTimerTick for TelemetryTimer {
    async fn tick(&self) {
        let to_write = {
            let write_mode = self.write_mode.get_write_mode();

            if write_mode.is_unknown() {
                self.detect_write_mode().await;
            }

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

        match self.write_mode.get_write_mode() {
            WriteMode::Unknown => {
                println!("Somehow we are unknown where to write telemetry");
            }
            WriteMode::Grpc => {
                if !self
                    .grpc_client
                    .write_events(self.app_name.as_str(), to_write)
                    .await
                {
                    self.write_mode.set_write_mode(WriteMode::Unknown);
                }
            }
            WriteMode::Http => {
                let url = self.settings.get_telemetry_url().await;
                if !crate::http_writer::write_as_http(
                    url.as_str(),
                    self.app_name.as_str(),
                    to_write,
                )
                .await
                {
                    self.write_mode.set_write_mode(WriteMode::Unknown);
                }
            }
        }
    }
}
