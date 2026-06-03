# 本地 RAG smoke 报告

- 索引: `/Users/leo/tmp-index-pull2/index.json`
- 模型: `/Users/leo/.arkui-rag/models/bge-m3`
- queries: `corpus/_eval/smoke-queries.yaml`
- top-K: 3
- 时间: 2026-06-03 12:24:48
- binary: `/Users/leo/.local/bin/arkui-rag` (arkui-rag 1.0.0)

---

## `a1` · ArkUI-X 怎么创建第一个应用

**期望**: 应命中 start-overview / start-with-ets-stage

**延迟**: 7861ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | README.md L4-9 | 快速开始 > 快速入门 |
| 2 | 0.0161 | start-overview.md L19-21 | 开发准备 > 开发工具 |
| 3 | 0.0159 | start-overview.md L2-6 | 开发准备 |

---

## `a2` · 什么是 ArkUI-X · 和 ArkUI 有什么区别

**期望**: 应命中 ArkUI-X-Overview / README

**延迟**: 6429ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | start-overview.md L14-17 | 开发准备 > 基本概念 > ArkUI-X |
| 2 | 0.0161 | sdk-structure-guide.md L4-6 | 简介 |
| 3 | 0.0159 | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |

---

## `a3` · DevEco Studio 怎么装 ArkUI-X 插件

**期望**: 应命中 start-with-deveco-studio / start-with-dev-environment

**延迟**: 6342ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | start-overview.md L23-27 | 开发准备 > 开发工具 > IDE工具（DevEco Studio） |
| 2 | 0.0161 | start-with-dev-environment.md L44-49 | 配置开发环境 > 安装ArkUI-X SDK |
| 3 | 0.0159 | start-with-dev-environment.md L4-13 | 配置开发环境 > 使用DevEco Studio开发ArkUI-X约束说明 |

---

## `a4` · ace tools 命令行工具用法

**期望**: 应命中 start-with-ace-tools

**延迟**: 5975ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | start-with-ace-tools.md L4-6 | ACE Tools快速指南 > 简介 |
| 2 | 0.0161 | start-with-ace-tools.md L94-95 | ACE Tools快速指南 > 常用命令参考 |
| 3 | 0.0159 | start-with-rom-size.md L8-10 | rom size 使用指导 > 2、rom size 环境相关配置 |

---

## `b1` · ArkUI-X 项目目录结构

**期望**: 应命中 package-structure-guide / sdk-structure-guide

**延迟**: 6021ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | sdk-structure-guide.md L4-6 | 简介 |
| 2 | 0.0161 | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |
| 3 | 0.0159 | start-with-dev-environment.md L44-49 | 配置开发环境 > 安装ArkUI-X SDK |

---

## `b2` · Stage 模型 ability 是什么

**期望**: 应命中 start-with-ets-stage / start-with-ability-on-android

**延迟**: 6179ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | start-with-ability-on-ios.md L10-12 | 通过Stage模型开发iOS端应用指南 > ArkUI-X和iOS平台集成所用关键类 > StageViewController |
| 2 | 0.0161 | start-with-ability-on-ios.md L34-36 | 通过Stage模型开发iOS端应用指南 > ArkUI-X和iOS平台集成所用关键类 > StageApplication |
| 3 | 0.0159 | start-with-ability-on-ios.md L14-26 | 通过Stage模型开发iOS端应用指南 > ArkUI-X和iOS平台集成所用关键类 > StageViewController > 公共属性 |

---

## `c1` · 怎么在 Android 上跑 ArkUI-X 应用

**期望**: 应命中 start-with-ability-on-android

**延迟**: 6984ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | sdk-structure-guide.md L4-6 | 简介 |
| 2 | 0.0161 | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |
| 3 | 0.0159 | start-overview.md L14-17 | 开发准备 > 基本概念 > ArkUI-X |

---

## `c2` · ArkUI-X iOS 端怎么集成

**期望**: 应命中 start-with-ability-on-ios

**延迟**: 7270ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | sdk-structure-guide.md L4-6 | 简介 |
| 2 | 0.0161 | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |
| 3 | 0.0159 | start-overview.md L14-17 | 开发准备 > 基本概念 > ArkUI-X |

---

## `c3` · ArkUI-X 不同平台行为差异

**期望**: 应命中 platform-different-introduction

**延迟**: 7078ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | start-overview.md L14-17 | 开发准备 > 基本概念 > ArkUI-X |
| 2 | 0.0161 | sdk-structure-guide.md L4-6 | 简介 |
| 3 | 0.0159 | package-structure-guide.md L4-6 | ArkUI-X应用工程结构说明 > 简介 |

---

## `d1` · platform bridge 怎么用 · 怎么调原生接口

**期望**: 应命中 platform-bridge-introduction

**延迟**: 7297ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | platform-different-introduction.md L10-15 | 平台差异化 > 使用场景及能力 > 使用场景 |
| 2 | 0.0161 | platform-bridge-introduction.md L72-74 | 平台桥接(@arkui-x.bridge) > 开发指南 |
| 3 | 0.0159 | start-with-ability-on-android.md L115-117 | 通过Stage模型开发Android端应用指南 > 通过原生Activity拉起Ability并传递参数 |

---

## `d2` · 动态化加载 · 热更新

**期望**: 应命中 dynamic-introduction

**延迟**: 6009ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | dynamic-introduction.md L14-17 | 动态化介绍 > 实践参考 |
| 2 | 0.0161 | start-with-ability-on-android.md L115-117 | 通过Stage模型开发Android端应用指南 > 通过原生Activity拉起Ability并传递参数 |
| 3 | 0.0159 | dynamic-introduction.md L4-12 | 动态化介绍 > 简介 |

---

## `d3` · ffi napi 调 C++ 接口

**期望**: 应命中 ffi-napi-introduction

**延迟**: 5837ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | ffi-napi-introduction.md L3-4 | FFI能力(N-API) > ArkUI-X中支持的N-API接口情况 |
| 2 | 0.0161 | ffi-napi-introduction.md L6-10 | FFI能力(N-API) > ArkUI-X中N-API的使用场景 |
| 3 | 0.0159 | start-with-ability-on-ios.md L14-26 | 通过Stage模型开发iOS端应用指南 > ArkUI-X和iOS平台集成所用关键类 > StageViewController > 公共属性 |

---

## `d4` · ArkTS 双向绑定 $$

**期望**: 应命中 arkts-two-way-sync

**延迟**: 6819ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | arkts-two-way-sync.md L2-9 | $$语法：内置组件双向同步 |
| 2 | 0.0161 | package-structure-guide.md L56-70 | ArkUI-X应用工程结构说明 > 编译构建说明 |
| 3 | 0.0159 | arkts-two-way-sync.md L11-18 | $$语法：内置组件双向同步 > 使用规则 |

---

## `e1` · 怎么访问 resource 资源

**期望**: 应命中 resource-categories-and-access

**延迟**: 6256ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | start-with-ability-on-android.md L115-117 | 通过Stage模型开发Android端应用指南 > 通过原生Activity拉起Ability并传递参数 |
| 2 | 0.0161 | start-with-ets-stage.md L10-12 | 使用ArkTS语言开发（Stage模型） > 应用介绍 |
| 3 | 0.0159 | resource-categories-and-access.md L2-7 | 资源分类与访问 |

---

## `f1` · 今天天气怎么样 · 北京下雨吗

**期望**: 无关 · 期望 top-1 score 极低

**延迟**: 6238ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | start-with-ability-on-ios.md L337-337 | xxx.ets > 2. WantParams工具类 |
| 2 | 0.0161 | start-with-ability-on-android.md L188-188 | xxx.ets > 2. WantParams工具类 |
| 3 | 0.0159 | start-with-ability-on-ios.md L300-307 | 通过Stage模型开发iOS端应用指南 > 通过iOS原生拉起Ability并传递参数 > 1. 使用手动方式 > 支持的参数类型列表 |

---

## `f2` · 怎么炒西红柿炒鸡蛋

**期望**: 无关 · 期望 top-1 score 极低

**延迟**: 6012ms

| # | score | source | heading |
|---|---|---|---|
| 1 | 0.0164 | start-with-ability-on-ios.md L337-337 | xxx.ets > 2. WantParams工具类 |
| 2 | 0.0161 | start-with-ability-on-android.md L188-188 | xxx.ets > 2. WantParams工具类 |
| 3 | 0.0159 | ios-slip-left-back.md L4-6 | IOS跨平台页面实现手势返回指南 > 简介 |

---

## 总览

- 总 queries: 16
- 成功执行: 16

