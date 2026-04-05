# Synoema — Build for macOS Apple Silicon (darwin-arm64)
#
# Usage:
#   make           Build synoema + synoema-mcp
#   make clean     Remove built binaries
#   make install   Copy to /usr/local/bin

TARGET   = aarch64-apple-darwin
LANG_DIR = ../../../lang
MCP_DIR  = ../../../mcp
PREFIX   = /usr/local

.PHONY: all clean install

all: synoema synoema-mcp

synoema:
	cargo build --release --target $(TARGET) -p synoema-repl --manifest-path $(LANG_DIR)/Cargo.toml
	cp $(LANG_DIR)/target/$(TARGET)/release/synoema ./synoema
	@echo "Built: synoema"

synoema-mcp:
	cargo build --release --target $(TARGET) --manifest-path $(MCP_DIR)/Cargo.toml
	cp $(MCP_DIR)/target/$(TARGET)/release/synoema-mcp ./synoema-mcp
	@echo "Built: synoema-mcp"

clean:
	rm -f synoema synoema-mcp

install: all
	install -d $(PREFIX)/bin
	install -m 755 synoema $(PREFIX)/bin/synoema
	install -m 755 synoema-mcp $(PREFIX)/bin/synoema-mcp
	@echo "Installed to $(PREFIX)/bin/"
