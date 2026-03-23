# KoPanel

KoPanel 是一款专为 GPD 掌机设计的控制面板，提供 TDP 调节、风扇控制、显示设置、手柄映射等丰富功能。

## 📥 下载

**下载主页**：[https://cdn.kobo07.cn/kopanel/index.html](https://cdn.kobo07.cn/kopanel/index.html)

## 💬 交流

**QQ 群**：178193167

## 📂 项目结构

本仓库包含 KoPanel 的**前端源码**（Svelte + TypeScript + Vite）。

```
src/                  # 前端源码
├── App.svelte        # 主组件
├── components/       # UI 组件
├── lib/              # 工具库 & 类型定义
├── styles/           # 样式文件
└── main.ts           # 入口
```

后端（Rust / Tauri）部分为闭源，不包含在本仓库中。

## 🛠️ 技术栈

- **前端框架**: [Svelte](https://svelte.dev/)
- **构建工具**: [Vite](https://vitejs.dev/)
- **桌面框架**: [Tauri](https://tauri.app/)（后端闭源）
- **语言**: TypeScript

## 📦 开发

```bash
# 安装依赖
npm install

# 启动开发服务器（仅前端）
npm run dev
```

> ⚠️ 由于后端部分未包含在此仓库，单独运行前端仅可预览 UI，无法使用硬件控制功能。

## 📄 License

前端代码采用 MIT License 开源。后端代码为闭源。
