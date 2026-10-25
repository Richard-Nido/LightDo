fn main() {
    println!("cargo:rerun-if-changed=assets/lightdo.rc");
    println!("cargo:rerun-if-changed=assets/lightdo.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("assets/lightdo.rc", embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }
}
