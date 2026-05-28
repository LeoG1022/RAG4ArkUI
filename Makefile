# RAG4ArkUI Makefile
# 目的：把常用 cargo 命令 + corpus 初始化收口到统一入口。
# 设计：所有 target 默认作用于 crates/ workspace。

CARGO ?= cargo
CRATES_DIR := crates

.PHONY: help install-rust check check-onnx check-tantivy check-treesitter check-lancedb build build-onnx build-tantivy build-treesitter build-lancedb build-full test fmt clippy clean corpus-init smoke

help:
	@echo "RAG4ArkUI — 可用 target"
	@echo "  make install-rust   提示安装 rust 工具链"
	@echo "  make check          cargo check --workspace（默认 features，不含 ONNX）"
	@echo "  make check-onnx     cargo check -p arkui-rag-embedding --features onnx"
	@echo "  make build          cargo build --workspace --release"
	@echo "  make build-onnx     cargo build -p arkui-rag-cli --features onnx --release (Day 3)"
	@echo "  make build-tantivy  cargo build -p arkui-rag-cli --features tantivy --release (Day 4)"
	@echo "  make build-full     cargo build -p arkui-rag-cli --features full --release（onnx + tantivy）"
	@echo "  make check-tantivy  cargo check -p arkui-rag-storage --features tantivy"
	@echo "  make check-treesitter  cargo check -p arkui-rag-chunker --features typescript (Day 10)"
	@echo "  make build-treesitter  cargo build CLI with tree-sitter ArkTS chunker (Day 10)"
	@echo "  make check-lancedb     cargo check -p arkui-rag-storage --features lancedb (Day 9)"
	@echo "  make build-lancedb     cargo build CLI with LanceDB vector store (Day 9)"
	@echo "  make test           cargo test --workspace"
	@echo "  make smoke          端到端冒烟：index + query 真实跑通（用 /tmp 临时 corpus）"
	@echo "  make fmt            cargo fmt --all"
	@echo "  make clippy         cargo clippy --workspace --all-targets"
	@echo "  make clean          cargo clean"
	@echo "  make corpus-init    确保 corpus/ 5 个子目录存在并提示用户投放文档"

install-rust:
	@command -v cargo >/dev/null 2>&1 && echo "✅ 已安装：$$(cargo --version)" || { \
	    echo "❌ 未检测到 cargo。请执行："; \
	    echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"; \
	    echo "  或："; \
	    echo "    brew install rustup-init && rustup-init"; \
	    exit 1; \
	}

check: install-rust
	cd $(CRATES_DIR) && $(CARGO) check --workspace

check-onnx: install-rust
	cd $(CRATES_DIR) && $(CARGO) check -p arkui-rag-embedding --features onnx

build: install-rust
	cd $(CRATES_DIR) && $(CARGO) build --workspace --release

build-onnx: install-rust
	cd $(CRATES_DIR) && $(CARGO) build -p arkui-rag-cli --features onnx --release

build-tantivy: install-rust
	cd $(CRATES_DIR) && $(CARGO) build -p arkui-rag-cli --features tantivy --release

build-full: install-rust
	cd $(CRATES_DIR) && $(CARGO) build -p arkui-rag-cli --features full --release

check-tantivy: install-rust
	cd $(CRATES_DIR) && $(CARGO) check -p arkui-rag-storage --features tantivy

check-treesitter: install-rust
	cd $(CRATES_DIR) && $(CARGO) check -p arkui-rag-chunker --features typescript

build-treesitter: install-rust
	cd $(CRATES_DIR) && $(CARGO) build -p arkui-rag-cli --features typescript --release

build-lancedb: install-rust
	cd $(CRATES_DIR) && $(CARGO) build -p arkui-rag-cli --features lancedb --release

check-lancedb: install-rust
	cd $(CRATES_DIR) && $(CARGO) check -p arkui-rag-storage --features lancedb

test: install-rust
	cd $(CRATES_DIR) && $(CARGO) test --workspace

smoke: install-rust
	bash scripts/demo-smoke.sh

fmt:
	cd $(CRATES_DIR) && $(CARGO) fmt --all

clippy:
	cd $(CRATES_DIR) && $(CARGO) clippy --workspace --all-targets -- -D warnings

clean:
	cd $(CRATES_DIR) && $(CARGO) clean

corpus-init:
	@mkdir -p corpus/official corpus/samples corpus/migration corpus/errors corpus/custom
	@echo "✅ corpus/ 5 个子目录就绪。请按 corpus/README.md 投放文档。"
