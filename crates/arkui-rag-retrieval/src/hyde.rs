//! HyDE (Hypothetical Document Embeddings) 改写器。
//!
//! 思路：用户输入"如何下拉刷新" → enhance 后产出"假代码"用做向量检索查询。
//! 向量空间里，"假代码"比原 query 更接近真实代码 chunk，召回精度提升。
//!
//! 技术方案对应：
//! - §2.4 检索流水线设计："HyDE 代码生成 → 先让 LLM 生成一段假代码"
//! - §6.2 模型 3：Query 改写小模型
//! - §1.2 Advanced RAG · Query 改写方向
//!
//! Day 7 状态：
//! - ✅ `MockHydeEnhancer`：确定性规则，无 LLM 依赖；让流水线先跑起来
//! - ⏳ `RemoteHydeEnhancer`：调远程 LLM 真实生成假代码，Week 3 单独切片接入

use arkui_rag_core::{
    query::{EnhancedQuery, QueryFilters, QueryIntent},
    QueryEnhancer, Result,
};
use async_trait::async_trait;

/// MockHyde —— 用确定性规则生成 ArkTS 风格"假代码"。
///
/// **算法**：
/// 1. 从 raw 中提取关键词（中英文混合）
/// 2. 检测意图（路由 / 列表 / 状态 / 错误 / 通用）
/// 3. 套对应的 ArkTS 模板，把 raw 嵌入注释
///
/// **价值**：
/// - 向量空间里"假代码"接近真实代码 chunk（如果用 OnnxEmbedder）
/// - MockEmbedder 阶段：让"以代码做查询"的链路提前打通
/// - 与 RemoteHyde 接口一致，未来切换零代码改动
pub struct MockHydeEnhancer {
    name: String,
}

impl Default for MockHydeEnhancer {
    fn default() -> Self {
        Self {
            name: "mock-hyde-arkts".to_string(),
        }
    }
}

impl MockHydeEnhancer {
    pub fn new() -> Self {
        Self::default()
    }
}

/// 简易意图分类（与 QueryIntent 对齐）。
fn classify_intent(raw: &str) -> QueryIntent {
    let lower = raw.to_lowercase();
    if lower.contains("router") || raw.contains("路由") || raw.contains("跳转") || raw.contains("页面") {
        QueryIntent::ApiLookup
    } else if lower.contains("error") || raw.contains("错误") || raw.contains("修复") || raw.contains("失败") {
        QueryIntent::ErrorFix
    } else if lower.contains("kmp") || lower.contains("android") || lower.contains("ios") || raw.contains("迁移") {
        QueryIntent::Migration
    } else if raw.contains("一多") || raw.contains("断点") || lower.contains("adaptive") {
        QueryIntent::Adaptive
    } else if raw.contains("组件") || lower.contains("component") || raw.contains("列表") || raw.contains("刷新") {
        QueryIntent::NewComponent
    } else {
        QueryIntent::Generic
    }
}

/// 简易实体抽取：取长度 >=2 的"代码/驼峰/方法"风格 token。
fn extract_entities(raw: &str) -> Vec<String> {
    let mut entities: Vec<String> = Vec::new();
    for tok in raw.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.') {
        if tok.len() < 2 {
            continue;
        }
        // 驼峰 / 含点 / 含数字的 token 视为代码标识符
        let is_code_like = tok.chars().any(|c| c.is_uppercase())
            || tok.contains('.')
            || tok.chars().any(|c| c.is_ascii_digit());
        if is_code_like {
            entities.push(tok.to_string());
        }
    }
    entities.dedup();
    entities
}

/// 按 intent 生成 ArkTS 风格"假代码"。
fn generate_hyde_code(raw: &str, intent: QueryIntent) -> String {
    match intent {
        QueryIntent::ApiLookup => format!(
            "// {raw}\nimport router from '@ohos.router';\n\n@Component\nstruct NavExample {{\n  build() {{\n    Button('跳转').onClick(() => {{\n      router.pushUrl({{ url: 'pages/Detail', params: {{ id: 1 }} }});\n    }})\n  }}\n}}",
            raw = raw
        ),
        QueryIntent::NewComponent => format!(
            "// {raw}\n@Component\nstruct ListExample {{\n  @State items: number[] = [1,2,3];\n  build() {{\n    Refresh({{ refreshing: false }}) {{\n      List() {{\n        ForEach(this.items, (it: number) => {{\n          ListItem() {{ Text(`item ${{it}}`) }}\n        }})\n      }}\n    }}\n  }}\n}}",
            raw = raw
        ),
        QueryIntent::ErrorFix => format!(
            "// {raw}\n// 编译错误 fix 示例\n@Component\nstruct FixedComp {{\n  // 1. 检查 @State / @Prop 装饰器位置\n  // 2. 确认 build() 返回类型\n  // 3. 校验 import 路径\n  build() {{}}\n}}",
            raw = raw
        ),
        QueryIntent::Migration => format!(
            "// {raw}\n// KMP/Android/iOS → ArkUI-X 迁移示例\n@Component\nstruct MigratedView {{\n  @State data: string = '';\n  aboutToAppear(): void {{\n    // 原 ViewModel.launch / lifecycleScope → async/await\n  }}\n  build() {{}}\n}}",
            raw = raw
        ),
        QueryIntent::Adaptive => format!(
            "// {raw}\n// 一多 / 断点适配示例\n@Component\nstruct AdaptiveLayout {{\n  @State breakpoint: string = 'sm';\n  build() {{\n    GridRow() {{\n      GridCol({{ span: {{ sm: 12, md: 6, lg: 4 }} }}) {{ Text('content') }}\n    }}\n  }}\n}}",
            raw = raw
        ),
        QueryIntent::Chitchat | QueryIntent::Generic => format!(
            "// {raw}\n@Component\nstruct Example {{\n  @State value: string = '';\n  build() {{\n    Column() {{\n      Text(this.value)\n    }}\n  }}\n}}",
            raw = raw
        ),
    }
}

#[async_trait]
impl QueryEnhancer for MockHydeEnhancer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn enhance(&self, raw: &str) -> Result<EnhancedQuery> {
        let intent = classify_intent(raw);
        let entities = extract_entities(raw);
        let hyde = generate_hyde_code(raw, intent);

        Ok(EnhancedQuery {
            raw: raw.to_string(),
            rewritten: raw.to_string(),
            hyde_doc: Some(hyde),
            entities,
            intent,
            filters: QueryFilters::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hyde_generates_arkts_template() {
        let e = MockHydeEnhancer::new();
        let q = e.enhance("ArkUI-X 怎么做下拉刷新").await.unwrap();
        assert_eq!(q.raw, "ArkUI-X 怎么做下拉刷新");
        assert!(q.hyde_doc.is_some());
        let hyde = q.hyde_doc.as_deref().unwrap();
        // ArkTS 风格代码必含的标识符
        assert!(hyde.contains("@Component"), "hyde 应含 @Component：{}", hyde);
        assert!(hyde.contains("build()"), "hyde 应含 build()：{}", hyde);
        // 原 query 应被嵌入注释（便于人工审计）
        assert!(hyde.contains("下拉刷新"));
    }

    #[tokio::test]
    async fn classifies_routing_intent() {
        let e = MockHydeEnhancer::new();
        let q = e.enhance("router.pushUrl 怎么传参数").await.unwrap();
        assert_eq!(q.intent, QueryIntent::ApiLookup);
        let hyde = q.hyde_doc.as_deref().unwrap();
        assert!(hyde.contains("router.pushUrl") || hyde.contains("@ohos.router"));
    }

    #[tokio::test]
    async fn classifies_migration_intent() {
        let e = MockHydeEnhancer::new();
        let q = e.enhance("KMP ViewModel 怎么迁移到 ArkUI-X").await.unwrap();
        assert_eq!(q.intent, QueryIntent::Migration);
    }

    #[tokio::test]
    async fn classifies_error_intent() {
        let e = MockHydeEnhancer::new();
        let q = e.enhance("@State 编译错误怎么修复").await.unwrap();
        assert_eq!(q.intent, QueryIntent::ErrorFix);
    }

    #[tokio::test]
    async fn extracts_code_like_entities() {
        let e = MockHydeEnhancer::new();
        let q = e.enhance("调用 router.pushUrl 传 RouterOptions 参数").await.unwrap();
        // 应至少识别出 "router.pushUrl" 和 "RouterOptions"
        assert!(q.entities.iter().any(|x| x.contains("router")));
        assert!(q.entities.iter().any(|x| x.contains("RouterOptions") || x.contains("Options")));
    }

    #[tokio::test]
    async fn deterministic_for_same_input() {
        let e = MockHydeEnhancer::new();
        let q1 = e.enhance("ArkUI-X").await.unwrap();
        let q2 = e.enhance("ArkUI-X").await.unwrap();
        assert_eq!(q1.hyde_doc, q2.hyde_doc);
    }
}
