use std::{sync::Arc, time::Duration};

use my_telemetry::TelemetryEvent;
use tokio::sync::Mutex;
use tonic::{transport::Channel, Request};

use crate::{
    writer_grpc::{
        telemetry_writer_client::TelemetryWriterClient, EventGrpcTag, TelemetryGrpcEvent,
    },
    MyTelemetrySettings,
};

const GRPC_TIMEOUT: Duration = Duration::from_secs(3);

pub struct GrpcClient {
    settings: Arc<dyn MyTelemetrySettings + Send + Sync + 'static>,
    channel: Mutex<Option<TelemetryWriterClient<Channel>>>,
}

impl GrpcClient {
    pub fn new(settings: Arc<dyn MyTelemetrySettings + Send + Sync + 'static>) -> Self {
        Self {
            channel: Mutex::new(None),
            settings,
        }
    }

    pub async fn is_grpc(&self) -> bool {
        let mut write_access = self.channel.lock().await;

        if let Some(channel) = write_access.as_mut() {
            return ping(channel).await;
        }

        let url = self.settings.get_telemetry_url().await;

        let telemetry_client = create_channel(url).await;

        if telemetry_client.is_none() {
            return false;
        }

        let mut telemetry_client = telemetry_client.unwrap();

        let result = ping(&mut telemetry_client).await;
        if result {
            *write_access = Some(telemetry_client);
        }

        result
    }

    pub async fn write_events(&self, service_name: &str, to_write: Vec<TelemetryEvent>) -> bool {
        let mut write_access = self.channel.lock().await;

        if write_access.is_none() {
            let channel = create_channel(self.settings.get_telemetry_url().await).await;
            if channel.is_none() {
                println!("Can not create MyTelemetry GRPC channel");
                return false;
            }

            *write_access = channel;
        }

        let grpc_channel = write_access.as_mut().unwrap();

        let mut grpc_items = Vec::with_capacity(to_write.len());

        for item in to_write {
            grpc_items.push(TelemetryGrpcEvent {
                process_id: item.process_id,
                started_at: item.started,
                finished_at: item.finished,
                service_name: service_name.to_string(),
                event_data: item.data,
                success: item.success,
                fail: item.fail,
                tags: if let Some(tags) = item.tags {
                    tags.into_iter()
                        .map(|x| EventGrpcTag {
                            key: x.key,
                            value: x.value,
                        })
                        .collect()
                } else {
                    vec![]
                },
            });
        }

        let future = grpc_channel.upload(futures::stream::iter(grpc_items));

        let result = tokio::time::timeout(GRPC_TIMEOUT, future).await;

        if result.is_err() {
            println!("Upload to GRPC channel timeout");
            *write_access = None;
            return false;
        }

        let result = result.unwrap();

        if result.is_err() {
            println!("Upload to GRPC channel had error");
            *write_access = None;
            return false;
        }

        true
    }
}

async fn create_channel(grpc_address: String) -> Option<TelemetryWriterClient<Channel>> {
    let grpc_channel = TelemetryWriterClient::connect(grpc_address);
    let result = tokio::time::timeout(GRPC_TIMEOUT, grpc_channel).await;

    if result.is_err() {
        return None;
    }

    let result = result.unwrap();

    if result.is_err() {
        return None;
    }

    let grpc_channel = result.unwrap();

    Some(grpc_channel)
}

async fn ping(channel: &mut TelemetryWriterClient<Channel>) -> bool {
    let feature = channel.ping(Request::new(()));

    let result = tokio::time::timeout(GRPC_TIMEOUT, feature).await;
    if result.is_err() {
        return false;
    }

    let result = result.unwrap();

    if result.is_err() {
        return false;
    }

    true
}
