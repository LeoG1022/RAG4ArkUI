# 本地 RAG smoke 报告

- 索引: `/Users/leo/tmp-index-pull2/index.json`
- 模型: `/Users/leo/.arkui-rag/models/bge-m3`
- queries: `corpus/_eval/smoke-queries.yaml`
- top-K: 3
- 时间: 2026-06-03 13:08:50
- binary: `/Users/leo/.local/bin/arkui-rag` (arkui-rag 1.0.0)

---

## `a1` · ArkUI-X 怎么创建第一个应用

**期望**: 应命中 start-overview / start-with-ets-stage

**延迟**: 12749ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.8161 | — | README.md L4-9 | 快速开始 > 快速入门 |
| 2 | 0.0161 | 0.8127 | — | start-overview.md L19-21 | 开发准备 > 开发工具 |
| 3 | 0.0159 | 0.8121 | — | start-overview.md L2-6 | 开发准备 |

---

## `a2` · 什么是 ArkUI-X · 和 ArkUI 有什么区别

**期望**: 应命中 ArkUI-X-Overview / README

**延迟**: 7825ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.8438 | — | start-overview.md L14-17 | 开发准备 > 基本概念 > ArkUI-X |
| 2 | 0.0161 | 0.8388 | — | sdk-structure-guide.md L4-6 | 简介 |
| 3 | 0.0159 | 0.8388 | — | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |

---

## `a3` · DevEco Studio 怎么装 ArkUI-X 插件

**期望**: 应命中 start-with-deveco-studio / start-with-dev-environment

**延迟**: 9887ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.8669 | — | start-overview.md L23-27 | 开发准备 > 开发工具 > IDE工具（DevEco Studio） |
| 2 | 0.0161 | 0.8539 | — | start-with-dev-environment.md L44-49 | 配置开发环境 > 安装ArkUI-X SDK |
| 3 | 0.0159 | 0.8438 | — | start-with-dev-environment.md L4-13 | 配置开发环境 > 使用DevEco Studio开发ArkUI-X约束说明 |

---

## `a4` · ace tools 命令行工具用法

**期望**: 应命中 start-with-ace-tools

**延迟**: 7502ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.8441 | — | start-with-ace-tools.md L4-6 | ACE Tools快速指南 > 简介 |
| 2 | 0.0161 | 0.8197 | — | start-with-ace-tools.md L94-95 | ACE Tools快速指南 > 常用命令参考 |
| 3 | 0.0159 | 0.8038 | — | start-with-rom-size.md L8-10 | rom size 使用指导 > 2、rom size 环境相关配置 |

---

## `b1` · ArkUI-X 项目目录结构

**期望**: 应命中 package-structure-guide / sdk-structure-guide

**延迟**: 6727ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.7799 | — | sdk-structure-guide.md L4-6 | 简介 |
| 2 | 0.0161 | 0.7799 | — | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |
| 3 | 0.0159 | 0.7667 | — | start-with-dev-environment.md L44-49 | 配置开发环境 > 安装ArkUI-X SDK |

---

## `b2` · Stage 模型 ability 是什么

**期望**: 应命中 start-with-ets-stage / start-with-ability-on-android

**延迟**: 6684ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.7694 | — | start-with-ability-on-ios.md L10-12 | 通过Stage模型开发iOS端应用指南 > ArkUI-X和iOS平台集成所用关键类 > StageViewController |
| 2 | 0.0161 | 0.7598 | — | start-with-ability-on-ios.md L34-36 | 通过Stage模型开发iOS端应用指南 > ArkUI-X和iOS平台集成所用关键类 > StageApplication |
| 3 | 0.0159 | 0.7396 | — | start-with-ability-on-ios.md L14-26 | 通过Stage模型开发iOS端应用指南 > ArkUI-X和iOS平台集成所用关键类 > StageViewController > 公共属性 |

---

## `c1` · 怎么在 Android 上跑 ArkUI-X 应用

**期望**: 应命中 start-with-ability-on-android

**延迟**: 8026ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.8460 | — | sdk-structure-guide.md L4-6 | 简介 |
| 2 | 0.0161 | 0.8460 | — | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |
| 3 | 0.0159 | 0.8349 | — | start-overview.md L14-17 | 开发准备 > 基本概念 > ArkUI-X |

---

## `c2` · ArkUI-X iOS 端怎么集成

**期望**: 应命中 start-with-ability-on-ios

**延迟**: 6888ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.8380 | — | sdk-structure-guide.md L4-6 | 简介 |
| 2 | 0.0161 | 0.8380 | — | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |
| 3 | 0.0159 | 0.8312 | — | start-overview.md L14-17 | 开发准备 > 基本概念 > ArkUI-X |

---

## `c3` · ArkUI-X 不同平台行为差异

**期望**: 应命中 platform-different-introduction

**延迟**: 7528ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.8514 | — | start-overview.md L14-17 | 开发准备 > 基本概念 > ArkUI-X |
| 2 | 0.0161 | 0.8283 | — | sdk-structure-guide.md L4-6 | 简介 |
| 3 | 0.0159 | 0.8283 | — | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |

---

## `d1` · platform bridge 怎么用 · 怎么调原生接口

**期望**: 应命中 platform-bridge-introduction

**延迟**: 6355ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.8260 | — | platform-different-introduction.md L10-15 | 平台差异化 > 使用场景及能力 > 使用场景 |
| 2 | 0.0161 | 0.8064 | — | platform-bridge-introduction.md L72-74 | 平台桥接(@arkui-x.bridge) > 开发指南 |
| 3 | 0.0159 | 0.7959 | — | start-with-ability-on-android.md L115-117 | 通过Stage模型开发Android端应用指南 > 通过原生Activity拉起Ability并传递参数 |

---

## `d2` · 动态化加载 · 热更新

**期望**: 应命中 dynamic-introduction

**延迟**: 7022ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.7293 | — | dynamic-introduction.md L14-17 | 动态化介绍 > 实践参考 |
| 2 | 0.0161 | 0.7179 | — | start-with-ability-on-android.md L115-117 | 通过Stage模型开发Android端应用指南 > 通过原生Activity拉起Ability并传递参数 |

---

## `d3` · ffi napi 调 C++ 接口

**期望**: 应命中 ffi-napi-introduction

**延迟**: 5886ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.7391 | — | ffi-napi-introduction.md L3-4 | FFI能力(N-API) > ArkUI-X中支持的N-API接口情况 |

---

## `d4` · ArkTS 双向绑定 $$

**期望**: 应命中 arkts-two-way-sync

**延迟**: 6748ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.7250 | — | arkts-two-way-sync.md L2-9 | $$语法：内置组件双向同步 |
| 2 | 0.0161 | 0.7058 | — | package-structure-guide.md L56-70 | ArkUI-X应用工程结构说明 > 编译构建说明 |
| 3 | 0.0159 | 0.7014 | — | arkts-two-way-sync.md L11-18 | $$语法：内置组件双向同步 > 使用规则 |

---

## `e1` · 怎么访问 resource 资源

**期望**: 应命中 resource-categories-and-access

**延迟**: 7957ms

| # | rrf | vector | bm25 | source | heading |
|---|---|---|---|---|---|
| 1 | 0.0164 | 0.7431 | — | start-with-ability-on-android.md L115-117 | 通过Stage模型开发Android端应用指南 > 通过原生Activity拉起Ability并传递参数 |
| 2 | 0.0161 | 0.7291 | — | start-with-ets-stage.md L10-12 | 使用ArkTS语言开发（Stage模型） > 应用介绍 |
| 3 | 0.0159 | 0.7273 | — | resource-categories-and-access.md L2-7 | 资源分类与访问 |

---

## `f1` · 今天天气怎么样 · 北京下雨吗

**期望**: 无关 · 期望 top-1 score 极低

**延迟**: 6825ms

⚠️ **无命中**（阈值过滤了所有结果 · 这通常是好事 = 负样本被剔）

---

## `f2` · 怎么炒西红柿炒鸡蛋

**期望**: 无关 · 期望 top-1 score 极低

**延迟**: 8509ms

⚠️ **无命中**（阈值过滤了所有结果 · 这通常是好事 = 负样本被剔）

---

## 总览

- 总 queries: 16
- 成功执行: 16

