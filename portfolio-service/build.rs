fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true) // generate server code
        .build_client(true) // generate client code (optional)
        .compile_protos(&["../proto/portfolio.proto"], &["../proto"])?;
    Ok(())
}
