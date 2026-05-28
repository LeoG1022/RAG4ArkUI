//! tree-sitter 切分公共工具。
//!
//! 把 tree-sitter Tree / Node 转换为 `Chunk` 列表的通用逻辑。
//! 各语言（TypeScript / Kotlin / Swift）只需提供"哪些 node kind 算 chunk"和
//! "如何从 node 取 name"两个策略。

#![cfg(feature = "treesitter")]

use arkui_rag_core::{Chunk, ChunkId, ChunkMetadata, ChunkType};
use tree_sitter::{Node, Tree};

/// 描述一个语言的切分策略。
pub(crate) trait LangStrategy {
    /// 哪些 node kind 应该被切成 chunk（top-level + class 内部 method 等）。
    fn interesting_kinds(&self) -> &'static [&'static str];

    /// 从 node 提取人类可读的 name（class 名 / method 名等）。
    fn extract_name(&self, node: Node, source: &str) -> Option<String>;

    /// 父 scope kind（如 class_declaration），用于 heading_path 链。
    fn is_scope_kind(&self, kind: &str) -> bool;
}

/// 切分入口：遍历 tree，按 strategy 收集 chunks。
///
/// Day 11：scope_stack 升级为 `(name, Option<ChunkId>)`，让 nested chunk 能找到父级 id。
pub(crate) fn extract_chunks<S: LangStrategy>(
    strategy: &S,
    tree: &Tree,
    source_path: &str,
    content: &str,
) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut scope_stack: Vec<(String, Option<ChunkId>)> = Vec::new();
    walk(strategy, tree.root_node(), source_path, content, &mut scope_stack, &mut chunks);
    chunks
}

fn walk<S: LangStrategy>(
    strategy: &S,
    node: Node,
    source_path: &str,
    content: &str,
    scope_stack: &mut Vec<(String, Option<ChunkId>)>,
    chunks: &mut Vec<Chunk>,
) {
    let kind = node.kind();
    let interested = strategy.interesting_kinds().contains(&kind);
    let is_scope = strategy.is_scope_kind(kind);

    // 若是 scope 节点，先 push 名字到 stack（影响下层 heading_path）
    let pushed = if is_scope || interested {
        if let Some(name) = strategy.extract_name(node, content) {
            scope_stack.push((name, None));
            true
        } else {
            false
        }
    } else {
        false
    };

    if interested {
        // 父 chunk_id = 栈倒数第 2 个（栈顶是"我"，所以父是 len-2）
        let parent_id = if scope_stack.len() >= 2 {
            scope_stack[scope_stack.len() - 2].1.clone()
        } else {
            None
        };
        let chunk = make_chunk(node, source_path, content, scope_stack, parent_id);
        // 写回栈顶 id，让后续 nested chunk 用作 parent
        if let Some(last) = scope_stack.last_mut() {
            last.1 = Some(chunk.id.clone());
        }
        chunks.push(chunk);
    }

    // 递归子节点
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(strategy, child, source_path, content, scope_stack, chunks);
    }

    if pushed {
        scope_stack.pop();
    }
}

fn make_chunk(
    node: Node,
    source_path: &str,
    content: &str,
    scope_stack: &[(String, Option<ChunkId>)],
    parent_id: Option<ChunkId>,
) -> Chunk {
    let start_byte = node.start_byte();
    let end_byte = node.end_byte();
    let text = &content[start_byte..end_byte];
    let start_line = node.start_position().row as u32 + 1;
    let end_line = node.end_position().row as u32 + 1;

    let heading_path: Vec<String> = scope_stack.iter().map(|(n, _)| n.clone()).collect();
    let id_path = if heading_path.is_empty() {
        format!("anonymous@{}", start_line)
    } else {
        heading_path.join("/")
    };
    let id = ChunkId::new(format!("{}#{}@{}", source_path, id_path, start_line));

    Chunk {
        id,
        content: text.to_string(),
        metadata: ChunkMetadata {
            source: source_path.to_string(),
            heading_path,
            line_range: Some((start_line, end_line)),
            r#type: ChunkType::CodeExample,
            parent_id,
            ..ChunkMetadata::default()
        },
    }
}

/// 通用 helper：取 node 第一个 `name` field（多数 tree-sitter 语法都有）。
pub(crate) fn name_by_field(node: Node, source: &str) -> Option<String> {
    if let Some(n) = node.child_by_field_name("name") {
        let s = n.start_byte();
        let e = n.end_byte();
        return Some(source[s..e].to_string());
    }
    None
}

/// 通用 helper：取 node 第一个 identifier 子节点。
pub(crate) fn name_by_first_identifier(node: Node, source: &str, kinds: &[&str]) -> Option<String> {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if kinds.contains(&child.kind()) {
            let s = child.start_byte();
            let e = child.end_byte();
            return Some(source[s..e].to_string());
        }
    }
    None
}
