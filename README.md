# PetContext - ADHD 友好的游戏化桌面宠物任务管理器

PetContext 是一款基于 Tauri 2.0 构建的桌面宠物应用，专为 ADHD 用户设计。它将任务管理与游戏化元素相结合，通过 Live2D 猫猫桌宠、AI 驱动的任务拆解、屏幕智能分析等功能，帮助用户降低执行阻力、保持专注。

## 核心功能

### 1. Live2D 桌面宠物
- 透明窗口、始终置顶的猫猫桌宠
- 支持拖拽移动，双击唤出对话窗口
- 气泡消息提醒（久坐提醒、截止日期提醒等）

### 2. AI 任务拆解（LLM）
- 接入 OpenAI 兼容 API（支持自定义 API 地址和模型）
- 基于 ADHD 执行功能辅助原则，将大任务拆解为原子级小步骤
- 游戏化设计：XP 经验值、能量等级、奖励建议
- 每个步骤包含具体的最小启动动作，降低心理门槛

### 3. VLM 屏幕智能分析
- Rust 原生截屏（xcap），无需 Python 依赖
- 支持 1-60 秒可配置的自动截屏间隔
- 截图本地保存，3 张轮换存储
- VLM（视觉语言模型）自动分析屏幕内容
- 自动提取待办事项和截止日期，添加到任务清单
- 情绪识别（工作/学习/娱乐/休息）

### 4. 智能提醒系统
- **久坐提醒**：可配置间隔（默认 45 分钟），8 条轮换健康提示
- **截止日期提醒**：三级预警（提前 48h/24h/4h），4 小时冷却避免重复打扰
- 气泡消息在桌宠窗口弹出

### 5. 任务视图
- **日视图**：当天任务列表及完成状态
- **周视图**：本周任务汇总
- **月视图**：本月任务概览
- 截止日期标签颜色编码（安全/提前/警告/紧急/已过期）

### 6. 日报与月报
- 自动统计鼠标点击数和键盘敲击数
- 结合 VLM 情绪分析、任务完成情况生成结构化报告
- LLM 生成叙事性总结
- 月报包含每日明细表格
- 支持导出为 PDF 和 Word (.docx) 文件

### 7. 其他特性
- 开机自启动（可在设置中开关）
- 聊天窗口智能定位（自动检测屏幕空间，避免窗口超出屏幕）
- 单实例运行保护
- 中文系统字体支持（PDF 导出自动加载系统中文字体）

## 技术栈

| 层级 | 技术 |
|------|------|
| 框架 | Tauri 2.0 |
| 前端 | Vue 3 + TypeScript + Vite |
| 后端 | Rust (tokio 异步运行时) |
| 桌宠渲染 | PixiJS + pixi-live2d-display |
| LLM/VLM | OpenAI 兼容 API (reqwest HTTP 客户端) |
| 截屏 | xcap (跨平台) |
| 图像处理 | image crate (JPEG 编码、缩放) |
| PDF 导出 | printpdf |
| Word 导出 | docx-rs |
| 输入监听 | rdev (全局鼠标/键盘事件) |
| 自动启动 | tauri-plugin-autostart |

## 项目结构

```
petcontext-finish-v1/
├── index.html                  # 主窗口入口
├── package.json                # Node.js 依赖
├── vite.config.ts              # Vite 构建配置
├── tsconfig.json               # TypeScript 配置
├── public/
│   ├── chat.html               # 对话窗口（任务/报告/洞察）
│   ├── js/                     # Live2D 运行时库
│   └── models/standard/        # Live2D 猫猫模型资源
├── src/
│   ├── App.vue                 # 主窗口 Vue 组件
│   ├── main.ts                 # 前端入口
│   ├── components/             # Vue 组件
│   └── utils/                  # 工具函数
└── src-tauri/
    ├── Cargo.toml              # Rust 依赖配置
    ├── tauri.conf.json         # Tauri 应用配置
    ├── capabilities/           # 权限声明
    ├── icons/                  # 应用图标
    └── src/
        ├── main.rs             # Rust 入口
        ├── lib.rs              # Tauri 命令注册与应用设置
        ├── llm.rs              # LLM API 调用与任务拆解
        ├── vlm.rs              # VLM 截屏分析（截图+多模态分析）
        ├── device.rs           # 全局输入监听（鼠标/键盘计数）
        └── report.rs           # PDF/Word 报告导出
```

## 开发环境搭建

### 前置要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) >= 1.77.2
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) v2

### 安装步骤

```bash
# 1. 安装 Node.js 依赖
npm install

# 2. 开发模式运行
npm run tauri dev

# 3. 构建发布版本
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录下。

## 使用指南

### 首次配置

1. 启动应用后，双击桌面猫猫唤出对话窗口
2. 点击右上角 **设置** 按钮
3. 配置 LLM API：
   - API 地址（如 `https://api.openai.com/v1` 或其他兼容服务）
   - API Key
   - 模型名称（如 `gpt-4o`、`deepseek-chat` 等）
4. 配置 VLM API（可选，用于屏幕分析）：
   - 同上，但选择支持多模态的模型（如 `gpt-4o`、`qwen-vl-plus` 等）
5. 点击 **测试连接** 确认配置正确

### 任务管理

- 在输入框输入任务描述，点击发送
- AI 自动将任务拆解为小步骤，每步附带 XP 奖励
- 勾选完成的步骤，积累经验值

### 屏幕分析

- 在设置中开启 **自动截屏分析**
- 调整截屏间隔（1-60 秒）
- VLM 自动识别屏幕内容、提取待办、判断工作状态
- 提取的任务自动添加到任务清单

### 报告

- 切换到 **报告** 标签页
- 选择日期/月份，点击 **生成报告**
- 查看统计数据、情绪分布、AI 叙事总结
- 点击 **导出 PDF** 或 **导出 Word** 保存

## 许可证

MIT License
