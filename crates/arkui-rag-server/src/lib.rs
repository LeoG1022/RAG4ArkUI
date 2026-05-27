#![doc = include_str!("../README.md")]

pub mod http;
pub mod lsp;
pub mod mcp;

/// 服务启动选项。`arkui-rag-cli` 的 `serve` subcommand 直接构造此结构传入。
#[derive(Debug, Clone, Default)]
pub struct ServeOptions {
    pub http: Option<HttpOptions>,
    pub mcp: bool,
    pub lsp: bool,
}

#[derive(Debug, Clone)]
pub struct HttpOptions {
    pub addr: std::net::SocketAddr,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1:7654".parse().expect("valid default addr"),
        }
    }
}
