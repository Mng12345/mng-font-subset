# 字体提取工具

[English](README_EN.md) | 中文

一个基于 Tauri + SolidJS 的桌面应用，用于从字体文件中提取指定字符的子集，有效减小字体文件体积。

## 功能特性

- **字体子集提取**：根据输入的文本内容，提取仅包含所需字符的字体文件
- **TTC 字体支持**：支持从 TrueType Collection 字体集合中提取指定索引的字体
- **实时预览**：输入文本后可预览字体渲染效果
- **跨平台**：支持 Windows、macOS 和 Linux

## 技术栈

- **前端**：SolidJS + TypeScript + Vite
- **后端**：Rust + Tauri
- **字体处理**：allsorts、ttf-parser

## 安装

### 开发环境要求

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://www.rust-lang.org/tools/install) 1.70+

### 安装依赖

```bash
npm install
```

## 开发

```bash
# 启动开发服务器
npm run tauri:dev
```

## 构建

```bash
# 构建当前平台的应用
npm run tauri:build

# 构建 Windows 版本
npm run tauri:build:win

# 构建 macOS 版本（需在 macOS 上执行）
npm run tauri:build:mac
```

构建完成后，安装包位于 `src-tauri/target/release/bundle/` 目录。

## 使用方法

1. 打开应用，点击"选择字体文件"上传字体（支持 TTF、OTF、TTC 格式）
2. 在文本输入框中输入需要保留的字符
3. 点击"提取字体"按钮
4. 选择保存位置，等待处理完成

## 项目结构

```
.
├── src/                    # 前端源代码 (SolidJS)
│   ├── components/         # 组件
│   ├── App.tsx            # 主应用组件
│   └── index.tsx          # 入口文件
├── src-tauri/             # Tauri 后端 (Rust)
│   ├── src/
│   │   └── lib.rs         # 核心字体处理逻辑
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
├── index.html             # HTML 模板
├── vite.config.ts         # Vite 配置
└── package.json           # Node.js 依赖
```

## 核心功能实现

### 字体子集提取

使用 Rust 的 `allsorts` 库进行字体子集提取：

1. 解析输入文本，提取唯一字符集合
2. 通过 cmap 表将字符映射为字形 ID
3. 使用 `SubsetProfile::Minimal` 进行最小化子集提取
4. 生成仅包含所需字形的新字体文件

### TTC 字体处理

支持从 TrueType Collection 中提取单个字体：

1. 解析 TTC 头部获取字体偏移量表
2. 根据指定索引提取对应字体的表数据
3. 重建独立的 TTF 文件结构
4. 重新计算校验和确保字体有效性

## 许可证

MIT
