#[async_trait::async_trait]
pub trait MyTelemetrySettings {
    async fn get_telemetry_settings(&self) -> String;
}
