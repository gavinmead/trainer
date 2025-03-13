# Makefile for C++23 project using Bazel

# Bazel settings
BAZEL = bazel
BAZEL_OUTPUT_USER_ROOT = $(shell pwd)/bazel-output
BAZEL_STARTUP_FLAGS = --output_user_root=$(BAZEL_OUTPUT_USER_ROOT)
BAZEL_BUILD_FLAGS = --enable_bzlmod --show_progress_rate_limit=1
BAZEL_TEST_FLAGS = --enable_bzlmod --test_output=errors

# Main targets
MAIN_TARGET = //src:main
TEST_TARGET = //tests:all_tests
ALL_TESTS = //...

# Directories
BUILD_DIR = build
BIN_DIR = $(BUILD_DIR)/bin

# Default target
all: build

# Create build directory
$(BUILD_DIR):
	mkdir -p $@

# Build the main program
build:
	$(BAZEL) $(BAZEL_STARTUP_FLAGS) build $(BAZEL_BUILD_FLAGS) $(MAIN_TARGET)
	@mkdir -p $(BIN_DIR)
	@cp -f bazel-bin/src/main $(BIN_DIR)/main

# Run the main program
run: build
	$(BIN_DIR)/main

# Run tests
test:
	$(BAZEL) $(BAZEL_STARTUP_FLAGS) test $(BAZEL_TEST_FLAGS) $(TEST_TARGET)

# Run all tests
test-all:
	$(BAZEL) $(BAZEL_STARTUP_FLAGS) test $(BAZEL_TEST_FLAGS) $(ALL_TESTS)

# Clean Bazel artifacts
clean:
	$(BAZEL) $(BAZEL_STARTUP_FLAGS) clean
	rm -rf $(BUILD_DIR)

# Clean everything, including Bazel cache
clean-all: clean
	rm -rf $(BAZEL_OUTPUT_USER_ROOT)

# Show info about Bazel setup
info:
	$(BAZEL) $(BAZEL_STARTUP_FLAGS) info

# Phony targets
.PHONY: all build run test test-all clean clean-all info

