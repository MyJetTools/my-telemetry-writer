mod grpc_writer;
mod my_telemetry_writer;
mod settings;
mod write_mode;
mod http_writer;
pub use my_telemetry_writer::MyTelemetryWriter;
pub use settings::MyTelemetrySettings;

pub mod writer_grpc {
    tonic::include_proto!("writer");
}
