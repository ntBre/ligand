fn main() {
    println!("cargo:rerun-if-env-changed=CONDA_PREFIX");
    let prefix = std::env::var("CONDA_PREFIX").unwrap();
    println!("cargo:rustc-link-search={prefix}/lib");
}
