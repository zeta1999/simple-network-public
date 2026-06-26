#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type NodeRunner;
        fn new_node_runner() -> Box<NodeRunner>;
        fn start(self: &mut NodeRunner, name: String);
    }
}

pub struct NodeRunner {
    #[allow(dead_code)]
    name: Option<String>,
}

pub fn new_node_runner() -> Box<NodeRunner> {
    Box::new(NodeRunner { name: None })
}

impl NodeRunner {
    pub fn start(&mut self, name: String) {
        self.name = Some(name.clone());
        println!("[CXX Bridge] NodeRunner started with name: {}", name);
    }
}
