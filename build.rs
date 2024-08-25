fn main() {
    tonic_build::compile_protos("protos/greet.proto")
        .unwrap_or_else(|e| panic!("Failed to compile greet protos {:?}", e));
}
