package main

/*
#cgo LDFLAGS: -L../target/debug -lsimple_network
#include <stdlib.h>

void simple_network_start_node(const char* name);
char* simple_network_get_version();
void simple_network_free_string(char* s);
*/
import "C"
import (
	"fmt"
	"unsafe"
)

func main() {
	versionCStr := C.simple_network_get_version()
	version := C.GoString(versionCStr)
	fmt.Printf("Simple Network Version: %s\n", version)
	C.simple_network_free_string(versionCStr)

	nodeName := C.CString("go_node_1")
	defer C.free(unsafe.Pointer(nodeName))
	
	C.simple_network_start_node(nodeName)
	fmt.Println("Go node started successfully.")
}
