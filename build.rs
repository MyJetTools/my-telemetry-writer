fn main() {
    let url = "https://raw.githubusercontent.com/MyJetTools/my-telemetry-server/main/proto/";
    ci_utils::sync_and_build_proto_file(url, "TelemetryWriter.proto");
}
