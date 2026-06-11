const PROTO_PATH: &str = "../../proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure().compile_protos(
        &[
            format!("{PROTO_PATH}/shinden_to_anilist/v1/service.proto"),
            format!("{PROTO_PATH}/shinden_to_anilist/v1/error.proto"),
            format!("{PROTO_PATH}/shinden_to_anilist/v1/search.proto"),
            format!("{PROTO_PATH}/shinden_to_anilist/v1/matching.proto"),
            format!("{PROTO_PATH}/shinden_to_anilist/v1/export.proto"),
        ],
        &[PROTO_PATH.into()],
    )?;
    Ok(())
}
