fn main() {
    println!("Simple Network Rust Demo");
    let mut runner = simple_network::ffi::cxx_bridge::new_node_runner();
    runner.start("rust_demo_node".to_string());
}
