// see https://github.com/PyO3/pyo3/issues/1576 and particularly
// https://github.com/PyO3/pyo3/issues/1576#issuecomment-1465231416
fn main() {
    let prefix = std::env::var("CONDA_PREFIX").unwrap();
    println!("cargo:rustc-env=LD_LIBRARY_PATH={prefix}/lib");
}
