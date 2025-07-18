BUILD_DIR 		  = ./build/debug
C_COMPILER_FLAG   = "-DCMAKE_C_COMPILER=C:/Program Files/LLVM/bin/clang.exe"
CPP_COMPILER_FLAG = "-DCMAKE_CXX_COMPILER=C:/Program Files/LLVM/bin/clang++.exe"

generate:
	cmake -S . -B $(BUILD_DIR) -G Ninja $(C_COMPILER_FLAG) $(CPP_COMPILER_FLAG)

compile: generate
	ninja -C$(BUILD_DIR)

run: compile
	$(BUILD_DIR)/core.exe