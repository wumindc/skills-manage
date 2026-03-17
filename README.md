# skills-manage

一个跨工具的 **Skill 本地管理器**（Rust 实现），支持：

- 从本地导入 skill 文件夹（需包含 `SKILL.md`）
- 统一存储在全局技能仓库
- 一键分发/同步到多种 AI 编程工具技能目录（Cursor / Claude Code / Codex / OpenCode / Antigravity / OpenClaw / Kiro CLI / Trae / Gemini CLI / Droid / Windsurf）
- 配置全局 skills 存储目录（带默认目录）
- 配置外部 skill 仓库源并拉取
- 搜索本地与外部源中的 skills
- 推荐热门 skills（基于本地 popularity 字段）
- GitHub Release 自动产出安装包：Windows、macOS Intel、macOS Apple Silicon（另附 Linux）

---

## 为什么选 Rust

- 单文件二进制，便于分发
- 性能稳定，适合大规模文件夹复制/同步
- 内存安全，适合长期运行的本地工具
- 跨平台编译链成熟，便于 CI 产出安装包

---

## 快速开始

```bash
cargo build --release
./target/release/skills-manage init
```

### 默认目录

- 配置目录：`~/.skills-manage/config.toml`
- 统一存储目录：`~/.skills-manage/store`
- 外部源缓存目录：`~/.skills-manage/sources`

---

## 常用命令

### 1) 初始化/修改全局存储目录

```bash
skills-manage init --global-store /data/skills-store
```

### 2) 导入本地 skill

```bash
skills-manage import /path/to/my-skill --id my-skill
```

> 未指定 `--id` 时，默认用文件夹名。

### 3) 注册工具目标目录

```bash
skills-manage register-tool cursor
skills-manage register-tool codex
skills-manage register-tool claude-code --path /custom/claude/skills
```

### 4) 分发/同步到所有已注册工具

```bash
skills-manage sync
```

### 5) 拉取外部 skill 仓库

```bash
skills-manage pull
```

### 6) 搜索 skill

```bash
skills-manage search agent
```

### 7) 热门推荐

```bash
skills-manage recommend
```

---

## 配置外部仓库（可编辑 config.toml）

`config.toml` 支持多个外部源：

```toml
[[sources]]
name = "awesome-ai-skills"
git_url = "https://github.com/example/awesome-ai-skills.git"
local_path = "/Users/you/.skills-manage/sources/awesome-ai-skills"
```

可按团队需要替换为企业私有仓库或社区仓库。

---

## GitHub Release 产物

项目内置 `.github/workflows/release.yml`：

- tag 推送 `v*` 自动触发
- 自动构建并上传：
  - `skills-manage-windows-x64.zip`
  - `skills-manage-macos-intel.tar.gz`
  - `skills-manage-macos-apple-silicon.tar.gz`
  - `skills-manage-linux-x64.tar.gz`

发布流程：

```bash
git tag v0.1.0
git push origin v0.1.0
```

然后到 GitHub Releases 下载对应安装包。

---

## 后续可扩展（建议）

- 文件监听（watch）+ 自动增量同步
- skill 元数据标准（`skill.toml`）与签名校验
- 热门推荐接入远程排行榜 API
- GUI（Tauri）桌面版安装器（DMG/MSI）
