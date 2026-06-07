const PROTO_PATH: &str = "../../proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure().compile_protos(
        &[format!("{PROTO_PATH}/shinden_to_anilist/v1/service.proto")],
        &[PROTO_PATH.into()],
    )?;
    Ok(())
}
