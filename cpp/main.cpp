#include "simple_network/src/ffi/cxx_bridge.rs.h"
#include <iostream>
#include <string>

int main() {
    auto runner = new_node_runner();
    runner->start(rust::String("cpp_node_1"));
    std::cout << "C++ node started successfully." << std::endl;
    return 0;
}
