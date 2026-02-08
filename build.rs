fn main() -> Result<(), anyhow::Error> {
    protobuf_codegen::Codegen::new()
        .include("protos")
        .inputs([
            "protos/OpenApiCommonMessages.proto",
            "protos/OpenApiCommonModelMessages.proto",
            "protos/OpenApiMessages.proto",
            "protos/OpenApiModelMessages.proto",
        ])
        .out_dir("src/proto_messages")
        .run()?;

    Ok(())
}
