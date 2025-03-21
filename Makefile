# Makefile for SQLiteCpp project - CMake wrapper

# Default build type
BUILD_TYPE ?= Release

# Default build directory
BUILD_DIR ?= build

# Debug build directory
DEBUG_BUILD_DIR ?= cmake-build-debug

# Debug build coverage directory
DEBUG_COV_BUILD_DIR ?= cmake-build-debug-coverage

# Install location (local by default)
INSTALL_PREFIX ?= $(CURDIR)/install

# Use internal SQLite by default
INTERNAL_SQLITE ?= ON

# Enable or disable coverage
COVERAGE ?= OFF

# Number of parallel jobs for building
JOBS ?= $(shell nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)

# Default targets
.PHONY: all
all: build

# Configure the project
.PHONY: configure
configure:
	@mkdir -p $(BUILD_DIR)
	@cd $(BUILD_DIR) && cmake .. \
		-DCMAKE_BUILD_TYPE=$(BUILD_TYPE) \
		-DENABLE_COVERAGE=$(COVERAGE)
	@echo "Configuration complete for build type $(BUILD_TYPE)"

# Build the project
.PHONY: build
build: configure
	@cmake --build $(BUILD_DIR) --config $(BUILD_TYPE) -j $(JOBS)
	@echo "Build complete"

# Install the library
.PHONY: install
install: build
	@cmake --install $(BUILD_DIR) --prefix $(INSTALL_PREFIX)
	@echo "Installation complete to $(INSTALL_PREFIX)"

# Run tests
.PHONY: test
test: build
	@cd $(BUILD_DIR) && ctest -C $(BUILD_TYPE) --output-on-failure
	@echo "Tests complete"

# Generate coverage report (only available when COVERAGE=ON)
.PHONY: coverage
coverage:
	$(MAKE) ENABLE_COVERAGE=ON BUILD_TYPE=Debug configure
	$(MAKE) build
	cd $(BUILD_DIR) && $(CMAKE) --build . --target coverage

# Create package
.PHONY: package
package: build
	@cd $(BUILD_DIR) && cpack
	@echo "Package creation complete"

# Clean build directory
.PHONY: clean
clean:
	@rm -rf $(BUILD_DIR)
	@rm -rf $(DEBUG_BUILD_DIR)
	@rm -rf $(DEBUG_COV_BUILD_DIR)
	@rm -rf lib
	@echo "Clean complete"

# Clean installed files
.PHONY: clean-install
clean-install:
	@rm -rf $(INSTALL_PREFIX)
	@echo "Installation directory cleaned"

# Full clean (build and install)
.PHONY: distclean
distclean: clean clean-install
	@echo "Full clean complete"