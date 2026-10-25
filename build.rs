fn main() {
    println!("cargo:rerun-if-changed=assets/lightdo.rc");
    println!("cargo:rerun-if-changed=assets/lightdo.manifest");
    println!("cargo:rerun-if-changed=assets/lightdo.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let root = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap()
            .replace('\\', "/");
        let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let version = std::env::var("CARGO_PKG_VERSION").unwrap();
        let commas = format!("{},0", version.replace('.', ","));
        let manifest = std::fs::read_to_string("assets/lightdo.manifest")
            .unwrap()
            .replace("@ASSEMBLY_VERSION@", &format!("{version}.0"));
        let manifest_path = out.join("lightdo.manifest");
        std::fs::write(&manifest_path, manifest).unwrap();
        let script = std::fs::read_to_string("assets/lightdo.rc")
            .unwrap()
            .replace("@VERSION_COMMAS@", &commas)
            .replace("@VERSION@", &version)
            .replace("\"lightdo.ico\"", &format!("\"{root}/assets/lightdo.ico\""))
            .replace(
                "\"lightdo.manifest\"",
                &format!(
                    "\"{}\"",
                    manifest_path.display().to_string().replace('\\', "/")
                ),
            );
        let path = out.join("lightdo.rc");
        std::fs::write(&path, script).unwrap();
        embed_resource::compile_for_everything(path, embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }
}
