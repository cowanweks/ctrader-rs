use tonic_prost_build::Config;

fn main() -> Result<(), anyhow::Error> {
    let mut config = Config::default();

    let messages = [
        "ProtoOaApplicationAuthReq",
        "ProtoOaAccountAuthReq",
        "ProtoOaRefreshTokenReq",
        "ProtoOaNewOrderReq",
        // Add all your message types
    ];

    for msg in &messages {
        config.type_attribute(msg, "#[serde(rename_all = \"camelCase\")]");
    }

    config
        .out_dir("src")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(
            &[
                "protos/OpenApiCommonMessages.proto",
                "protos/OpenApiCommonModelMessages.proto",
                "protos/OpenApiMessages.proto",
                "protos/OpenApiModelMessages.proto",
            ],
            &["protos/"],
        )?;

    println!("cargo:rerun-if-changed=protos/");

    Ok(())
}
