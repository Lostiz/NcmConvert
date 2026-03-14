# NcmConvert

网易云音乐 NCM 格式音频转换工具，支持将 NCM 格式转换为 MP3、FLAC 等常见音频格式。

## 功能特性

- 支持 NCM 格式解密转换
- 支持 MP3、FLAC 输出格式
- 支持批量文件转换
- 支持拖拽文件操作
- 自动保留音频元数据（封面、歌词、艺术家等）

## 技术栈

- [Tauri 2.0](https://tauri.app/) - 跨平台桌面应用框架
- [Vue 3](https://vuejs.org/) - 前端框架
- [TypeScript](https://www.typescriptlang.org/) - 类型安全
- [Naive UI](https://www.naiveui.com/) - UI 组件库
- [FFmpeg](https://ffmpeg.org/) - 音频处理

## 开发环境

### 前置要求

- Node.js >= 18
- Rust >= 1.70
- pnpm / npm / yarn

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

### 构建发布

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 项目结构

```
NcmConvert/
├── src/                    # Vue 前端源码
│   ├── components/         # Vue 组件
│   ├── App.vue            # 根组件
│   └── main.ts            # 入口文件
├── src-tauri/             # Tauri 后端源码
│   ├── src/               # Rust 源码
│   │   ├── main.rs        # 主程序入口
│   │   └── ncm.rs         # NCM 解密模块
│   ├── icons/             # 应用图标
│   ├── resources/         # 资源文件 (ffmpeg)
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
├── public/                # 静态资源
├── index.html             # HTML 入口
├── package.json           # npm 配置
├── vite.config.ts         # Vite 配置
└── tsconfig.json          # TypeScript 配置
```

## 使用说明

1. 打开应用程序
2. 点击"选择文件"或直接拖拽 NCM 文件到窗口
3. 选择输出格式（MP3 或 FLAC）
4. 选择输出目录
5. 点击"开始转换"
6. 转换完成后可在输出目录查看文件

## 注意事项

- 确保系统已安装 FFmpeg 工具，且可在命令行中调用 `ffmpeg` 命令
- 建议使用最新版本的 FFmpeg，以获得最佳性能和兼容性


## 免责声明

- 本工具仅供个人学习和研究使用
- 严禁将本工具用于任何商业目的或非法用途
- 请勿用于商业用途或大规模传播
- 本工具的目的是解析 ncm 加密文件的格式与算法，以促进技术交流。请尊重版权，支持正版音乐
- 因使用本工具而产生的一切法律后果及责任，均由使用者本人承担。作者概不负责，亦不承担任何法律责任

## 许可证

[MIT License](LICENSE)

## 致谢

- [anonymous5l](https://github.com/anonymous5l) - NCM 格式解密算法原始实现（Go）（已删库）
- [taurusxin](https://github.com/taurusxin/ncmdump) - NCM 格式解密参考
