# 小说爬虫工具使用说明

本工具包含一个独立的 CLI 命令行工具，用于从起点中文网和独阅读网站爬取小说元数据。

## 功能特性

- 支持起点中文网和独阅读网站
- 自动提取小说元数据（作者、简介、分类、封面）
- 支持直接输入起点书籍 URL
- 支持 JSON 格式输出到文件
- 使用 headless Chrome 绕过反爬虫检测

## 安装与构建

### 方法一：使用预编译脚本（推荐）

#### Windows 用户
```batch
cd tools
run-scraper.bat 起点 "诡秘之主"
```

#### Linux/Mac 用户
```bash
cd tools
chmod +x run-scraper.sh
./run-scraper.sh 起点 "诡秘之主"
```

### 方法二：直接使用 Cargo

```bash
cd src-tauri
cargo run --bin scraper_cli -- 起点 "诡秘之主"
```

## 使用方法

### 基本语法

```bash
cargo run --bin scraper_cli -- <来源站点> <书名> [作者] [输出文件]
```

### 参数说明

- **来源站点**（必需）：`起点` 或 `独阅读`
- **书名**（必需）：要搜索的小说名称，或直接输入起点 URL
- **作者**（可选）：用于精确匹配
- **输出文件**（可选）：将结果保存为 JSON 文件

### 使用示例

#### 1. 搜索起点小说（仅书名）
```bash
cargo run --bin scraper_cli -- 起点 "诡秘之主"
```

#### 2. 搜索起点小说（书名 + 作者）
```bash
cargo run --bin scraper_cli -- 起点 "诡秘之主" "爱潜水的乌贼"
```

#### 3. 使用起点 URL 直接获取
```bash
cargo run --bin scraper_cli -- 起点 "https://www.qidian.com/book/1010400217"
```

#### 4. 搜索并保存为 JSON
```bash
cargo run --bin scraper_cli -- 起点 "诡秘之主" "爱潜水的乌贼" output.json
```

#### 5. 搜索独阅读网站
```bash
cargo run --bin scraper_cli -- 独阅读 "某部小说"
```

## 输出格式

### 终端输出

```
🕷️  小说爬虫工具 v1.0
===================

📚 来源站点: 起点
📖 书名: 诡秘之主
✍️  作者: 爱潜水的乌贼

🌐 开始爬取...
✅ 爬取成功！

📋 元数据:
  ✍️  作者: 爱潜水的乌贼
  📂 分类: 奇幻
  📝 简介:
     螺旋塔是一部奇幻小说...
  🖼️  封面: https://bookcover.yuewen.com/qdbimg/349573/1010400217
```

### JSON 输出（当指定输出文件时）

```json
{
  "source_site": "起点",
  "title": "诡秘之主",
  "author": "爱潜水的乌贼",
  "category": "奇幻",
  "description": "...",
  "cover_url": "https://..."
}
```

## 技术细节

### 反爬虫策略

- 使用 headless Chrome 模拟真实浏览器访问
- 自动设置 User-Agent 和 Referer
- 支持多种页面选择器模式以适应网站变化
- 自动 cookie 处理

### 错误处理

- 网络超时：自动重试
- JSON 解析失败：回退到 HTML 解析
- 未找到书籍：提供详细的错误提示和建议

### 性能优化

- 异步 I/O 操作
- 连接池复用
- 图片自动压缩（最大宽度 800px）

## 常见问题

### Q: 构建失败怎么办？
A: 确保已安装 Rust 和 Cargo，并且网络连接正常。可以尝试：
```bash
cargo clean
cargo build --bin scraper_cli
```

### Q: 爬取失败怎么办？
A: 可能的原因：
1. 网络连接问题
2. 目标网站反爬虫策略更新
3. 书名或作者名称不准确

建议：
- 使用完整的起点 URL 而非书名搜索
- 确保作者名称准确
- 检查网络连接

### Q: 如何添加新的小说网站支持？
A: 在 `src-tauri/src/modules/novel/scraper.rs` 中参考现有的 `scrape_qidian` 和 `scrape_duyuedu` 函数添加新的爬虫函数。

## 开发与调试

### 运行测试
```bash
cd src-tauri
cargo test scrape
```

### 查看详细日志
设置环境变量 `RUST_LOG=debug`：
```bash
RUST_LOG=debug cargo run --bin scraper_cli -- 起点 "测试"
```

## 许可证

MIT License - 仅供学习和研究使用，请遵守目标网站的 robots.txt 和使用条款。
