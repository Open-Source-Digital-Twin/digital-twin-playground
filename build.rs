fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "grpc")]
    {
        let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
        tonic_build::configure()
            .file_descriptor_set_path(out_dir.join("digital_twin_descriptor.bin"))
            .compile_protos(&["proto/digital_twin.proto"], &["proto"])?;
    }
    Ok(())
}
