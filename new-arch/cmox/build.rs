fn main() {
    println!("cargo::rustc-link-arg-tests=-Tembedded-test.x");
    println!("cargo::rustc-link-arg-tests=-Tdefmt.x");
}
