extern crate protoc_rust;

fn main() {
    protoc_rust::Codegen::new()
        .out_dir("src/")
        .inputs(&["src/geoip.proto"])
        .include(".")
        .run()
        .expect("protoc");
}
