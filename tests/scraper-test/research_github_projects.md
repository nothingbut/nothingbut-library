# GitHub起点爬虫项目调研

## 调研目标
搜索GitHub上专门针对起点中文网的爬虫项目，学习成熟方案。

## 搜索关键词
- "qidian crawler"
- "qidian spider"
- "起点爬虫"
- "起点小说下载"
- "qidian scraper"
- "novel downloader qidian"

## 发现的项目

### 1. Novel-Downloader (404-novel-project)
**链接**: https://github.com/404-novel-project/novel-downloader
**语言**: TypeScript
**技术方案**:
- ✅ 浏览器用户脚本（Tampermonkey/Greasemonkey）
- ✅ 直接在浏览器环境中运行
- ✅ 支持100+小说网站（包括起点）
- ✅ 使用网站本身的登录状态和cookies
- ✅ 支持VIP章节（需要token）

**关键发现**：
```typescript
// Token配置示例
const tokenOptions = {
  Jjwxc: {
    token: "12345678_abcdef1234567890abcdef",
    user_key: "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
  }
};
window.tokenOptions = tokenOptions;
```

**优点**：
- 在真实浏览器环境运行，不会被识别为bot
- 利用用户的登录状态
- 能访问VIP章节

**缺点**：
- 需要浏览器环境
- 需要用户安装脚本
- 不适合我们的Tauri桌面应用

**启发**：
- 💡 可以考虑在Tauri应用中嵌入WebView，运行用户脚本
- 💡 或者让用户在浏览器中完成验证后，把cookies导入应用

---

### 2. 需要继续调研的项目

搜索更多项目中...

**搜索方向**：
1. Python实现的起点爬虫
2. Go实现的小说下载器
3. Rust实现的爬虫（最理想）
4. 起点的非官方API文档

**期望找到**：
- ✅ 起点的API endpoint列表
- ✅ 绕过WAF的具体技术
- ✅ Cookie/Token管理方案
- ✅ 请求头配置
- ✅ 代理使用方案

---

## 可能的技术方案（基于调研）

### 方案1: 嵌入式浏览器方案
**灵感来源**: Novel-Downloader

**实现思路**：
1. 在Tauri应用中使用WebView
2. 让用户在WebView中登录起点
3. 注入JavaScript获取书籍信息
4. 提取数据到Rust后端

**优点**：
- ✅ 利用真实浏览器环境
- ✅ 可以使用用户登录状态
- ✅ 不会被识别为bot

**缺点**：
- ⚠️ 需要用户登录
- ⚠️ 实现复杂度高

### 方案2: Cookie导入方案
**灵感来源**: 浏览器开发工具

**实现思路**：
1. 提示用户在浏览器中访问起点
2. 用户完成登录和验证
3. 用户导出cookies（提供工具）
4. 应用使用cookies访问API

**优点**：
- ✅ 绕过WAF验证
- ✅ 可以访问登录后的内容

**缺点**：
- ⚠️ 用户操作步骤较多
- ⚠️ Cookies会过期

### 方案3: 寻找移动端API
**搜索方向**: 起点的移动端API

**假设**：
- 起点的移动APP肯定有API
- 可能更容易绕过WAF
- 需要找到API endpoint和签名方式

**TODO**：
- 抓包分析起点APP的网络请求
- 分析API签名算法
- 测试API的可用性

---

## 继续调研计划

### 第一步：搜索更多GitHub项目
需要在GitHub上搜索：
- qidian
- 起点
- qidian api
- qidian chapter

### 第二步：分析移动端APP
使用工具：
- Charles Proxy
- mitmproxy
- Wireshark

### 第三步：查找技术文章
搜索关键词：
- "起点爬虫技术"
- "起点API分析"
- "qidian scraping tutorial"

---

## 待验证的想法

### 想法1: 起点APP的API
- 📱 起点有官方APP
- 🔍 APP肯定使用JSON API
- 💡 可能比Web端更容易访问

**验证方法**：
- 安装起点APP
- 使用Charles抓包
- 分析API结构

### 想法2: 图片OCR方案
- 📷 有些网站用图片防爬
- 🤖 Novel-Downloader支持OCR
- 💡 如果起点用图片，可以OCR识别

**技术栈**：
- PaddleOCR
- Tesseract

### 想法3: WebView注入方案
- 🌐 Tauri支持WebView
- 💉 可以注入JavaScript
- 💡 在WebView中访问起点，提取数据

**参考**：
- Tauri可以与WebView交互
- 使用消息传递机制获取数据

---

## 下一步行动

1. ⬜ 在GitHub搜索更多起点相关项目
2. ⬜ 分析起点APP的API（如果可行）
3. ⬜ 测试WebView方案的可行性
4. ⬜ 实现Cookie导入功能
5. ⬜ 编写概念验证代码

---

## 参考资料

- Novel-Downloader: https://github.com/404-novel-project/novel-downloader
- Tauri WebView文档: https://tauri.app/v1/guides/features/webview
- PaddleOCR: https://github.com/PaddlePaddle/PaddleOCR
