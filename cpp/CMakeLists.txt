cmake_minimum_required(VERSION 3.10)
project(simple_network_cpp)

set(CMAKE_CXX_STANDARD 17)

add_executable(simple_network_demo main.cpp)

# Link against the Rust static library built by cargo
target_link_libraries(simple_network_demo 
    PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../target/debug/libsimple_network.a
    -lpthread -ldl -lm
)

if(APPLE)
    target_link_libraries(simple_network_demo PRIVATE "-framework CoreFoundation" "-framework Security")
endif()
