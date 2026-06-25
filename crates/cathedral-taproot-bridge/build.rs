use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_root = PathBuf::from("proto");

    // Arquivos proto do tapd
    let proto_files = [
        "proto/taprootassets.proto",
        "proto/assetwallet.proto",
        "proto/universe.proto",
        "proto/mint.proto",
        "proto/rfq.proto",
        "proto/tapcommon.proto",
    ];

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile_protos(&proto_files, &[&proto_root])?;

    Ok(())
}
