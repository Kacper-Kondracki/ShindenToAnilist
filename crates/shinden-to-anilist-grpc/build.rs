const PROTO_PATH: &str = "../../proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    emit_build_revision();
    tonic_prost_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(
            &[
                format!("{PROTO_PATH}/shinden_to_anilist/v1/service.proto"),
                format!("{PROTO_PATH}/shinden_to_anilist/v1/error.proto"),
                format!("{PROTO_PATH}/shinden_to_anilist/v1/search.proto"),
                format!("{PROTO_PATH}/shinden_to_anilist/v1/matching.proto"),
                format!("{PROTO_PATH}/shinden_to_anilist/v1/export.proto"),
                format!("{PROTO_PATH}/shinden_to_anilist/v1/source.proto"),
            ],
            &[PROTO_PATH.into()],
        )?;
    Ok(())
}

fn emit_build_revision() {
    fn git(args: &[&str]) -> Option<String> {
        let output = std::process::Command::new("git").args(args).output().ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    for reference in [
        Some("HEAD".to_owned()),
        git(&["symbolic-ref", "-q", "HEAD"]),
        Some("packed-refs".to_owned()),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(path) = git(&["rev-parse", "--git-path", &reference]) {
            let path = std::path::Path::new(&path);
            if path.exists() {
                println!("cargo:rerun-if-changed={}", path.display());
            } else if let Some(parent) = path.parent().filter(|parent| parent.exists()) {
                if reference.starts_with("refs/") {
                    println!("cargo:rerun-if-changed={}", parent.display());
                }
            }
        }
    }
    let revision = git(&["rev-parse", "--short=12", "HEAD"]).unwrap_or_else(|| "unknown".to_owned());
    println!("cargo:rustc-env=SHINDEN_BUILD_REVISION={revision}");
}
