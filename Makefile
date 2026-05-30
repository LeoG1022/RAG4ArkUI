# RAG4ArkUI Makefile
# 目的：把常用 cargo 命令 + corpus 初始化收口到统一入口。
# 设计：所有 target 默认作用于 crates/ workspace。

CARGO ?= cargo
CRATES_DIR := crates

.PHONY: help install-rust check check-onnx check-tantivy check-treesitter check-lancedb check-http check-mcp check-lsp build build-onnx build-tantivy build-treesitter build-lancedb build-http build-mcp build-lsp build-full test fmt clippy clean corpus-init smoke serve-demo serve-mcp-demo serve-lsp-demo mcp-demo release-local release-local-verify book-build book-serve book-clean install-mdbook

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
	@echo "  make check-http        cargo check -p arkui-rag-server --features http (Day 14)"
	@echo "  make build-http        cargo build CLI with HTTP server (Day 14)"
	@echo "  make serve-demo        启动 demo HTTP server (127.0.0.1:7654)"
	@echo "  make check-mcp         cargo check -p arkui-rag-server --features mcp (Day 15)"
	@echo "  make build-mcp         cargo build CLI with MCP stdio server (Day 15)"
	@echo "  make serve-mcp-demo    启动 MCP stdio server"
	@echo "  make mcp-demo          MCP 端到端演示（启动 server + 喂 4 请求 + 断言响应 · Day 19）"
	@echo "  make check-lsp         cargo check -p arkui-rag-server --features lsp (Day 16)"
	@echo "  make build-lsp         cargo build CLI with LSP stdio server (Day 16)"
	@echo "  make serve-lsp-demo    启动 LSP stdio server"
	@echo "  make test           cargo test --workspace"
	@echo "  make smoke          端到端冒烟：index + query 真实跑通（用 /tmp 临时 corpus）"
	@echo "  make fmt            cargo fmt --all"
	@echo "  make clippy         cargo clippy --workspace --all-targets"
	@echo "  make clean          cargo clean"
	@echo "  make corpus-init    确保 corpus/ 5 个子目录存在并提示用户投放文档"
	@echo "  make release-local         本地打 release tarball 到 dist/ (Day 20)"
	@echo "  make release-local-verify  打包 + 解压 + 跑 --version 验证（Day 20）"
	@echo "  make book-build            构建 mdBook 文档站到 mdbook/book/ (Day 22)"
	@echo "  make book-serve            mdbook serve 本地预览（http://localhost:3000）"
	@echo "  make book-clean            清 mdbook/book/"
	@echo "  make install-mdbook        检查 / 提示安装 mdbook（brew / cargo）"

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

check-http: install-rust
	cd $(CRATES_DIR) && $(CARGO) check -p arkui-rag-server --features http

build-http: install-rust
	cd $(CRATES_DIR) && $(CARGO) build -p arkui-rag-cli --features http --release

serve-demo: install-rust
	cd $(CRATES_DIR) && $(CARGO) run -p arkui-rag-cli --features http -- \
	    serve --http --addr 127.0.0.1:7654

check-mcp: install-rust
	cd $(CRATES_DIR) && $(CARGO) check -p arkui-rag-server --features mcp

build-mcp: install-rust
	cd $(CRATES_DIR) && $(CARGO) build -p arkui-rag-cli --features mcp --release

serve-mcp-demo: install-rust
	cd $(CRATES_DIR) && $(CARGO) run -p arkui-rag-cli --features mcp -- \
	    serve --mcp

mcp-demo: install-rust
	bash scripts/mcp-demo.sh

check-lsp: install-rust
	cd $(CRATES_DIR) && $(CARGO) check -p arkui-rag-server --features lsp

build-lsp: install-rust
	cd $(CRATES_DIR) && $(CARGO) build -p arkui-rag-cli --features lsp --release

serve-lsp-demo: install-rust
	cd $(CRATES_DIR) && $(CARGO) run -p arkui-rag-cli --features lsp -- \
	    serve --lsp

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

# Day 20: 本地 release artifact（host 平台 · 跨平台 CI matrix 留 Day 20 续）
release-local: install-rust
	bash scripts/release-local.sh

# 打包 + 解压 + 跑 --version + query 自验证
release-local-verify: release-local
	@echo ""
	@echo "━━━ 解压验证 ━━━"
	@rm -rf /tmp/arkui-rag-release-verify && mkdir -p /tmp/arkui-rag-release-verify
	@tar -xzf dist/arkui-rag-v0.0.1-$$(rustc -vV | awk '/^host:/ {print $$2}').tar.gz -C /tmp/arkui-rag-release-verify
	@/tmp/arkui-rag-release-verify/arkui-rag-v0.0.1-$$(rustc -vV | awk '/^host:/ {print $$2}')/arkui-rag --version
	@echo "✅ release tarball 端到端可用"

# Round 37: 一键装到 ~/.local/bin/ + 自动配 Claude CLI / Desktop MCP（不用 sudo · 避开 macOS provenance）
install: release-local
	bash scripts/install-binary.sh

# 安装但跳过 MCP 自动配置（适合 CI · 或已用 claude mcp add 配过）
install-no-mcp: release-local
	bash scripts/install-binary.sh --skip-mcp

# Round 39: 反向操作 · 删 binary + 移除三端 MCP 配置（默认 dry-run · 加 ARGS=--yes 真删）
uninstall:
	bash scripts/uninstall-binary.sh $(ARGS)

# 真删（不需要 ARGS=--yes 这种写法时用）· 直接 make uninstall-yes
uninstall-yes:
	bash scripts/uninstall-binary.sh --yes

# Day 22: mdBook 文档站
install-mdbook:
	@command -v mdbook >/dev/null 2>&1 && echo "✅ 已安装：$$(mdbook --version)" || { \
	    echo "❌ 未检测到 mdbook。请执行："; \
	    echo "    brew install mdbook"; \
	    echo "  或："; \
	    echo "    cargo install mdbook --locked"; \
	    exit 1; \
	}

book-build: install-mdbook
	cd mdbook && mdbook build
	@echo ""
	@echo "✅ 站点输出：mdbook/book/index.html"
	@echo "   推 master 触发 .github/workflows/book.yml 自动部署到 gh-pages"

book-serve: install-mdbook
	cd mdbook && mdbook serve --open

book-clean:
	rm -rf mdbook/book
