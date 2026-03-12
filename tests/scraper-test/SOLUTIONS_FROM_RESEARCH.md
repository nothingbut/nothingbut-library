# 从网络调研获得的解决方案

基于GitHub项目调研和技术文章分析，总结可行的起点爬虫方案。

---

## 🔍 调研发现的关键项目

### 1. Novel-Downloader（404-novel-project）
**GitHub**: https://github.com/404-novel-project/novel-downloader
**Star数**: 1k+
**技术栈**: TypeScript + Tampermonkey
**支持网站**: 100+（包括起点、晋江、17K等）

**核心思路**：
- ✅ 在**真实浏览器环境**中运行（用户脚本）
- ✅ 利用用户的**登录状态和cookies**
- ✅ 不会被识别为bot（因为就是在真实浏览器中）
- ✅ 支持VIP章节（使用用户token）

**关键代码**：
```typescript
// 在浏览器环境中运行，直接访问DOM
const bookInfo = document.querySelector('.book-info');
const bookName = bookInfo.querySelector('h1').innerText;
const author = bookInfo.querySelector('.writer').innerText;

// 使用浏览器的fetch，自动携带cookies
fetch('https://www.qidian.com/ajax/search?kw=xxx')
  .then(res => res.json())
  .then(data => console.log(data));
```

**为什么成功**：
1. 在真实浏览器中，不是headless
2. 有完整的浏览器指纹
3. 自动携带用户的登录状态
4. 用户已经完成了验证码

---

## 💡 可行的技术方案

基于调研，我们有以下几种可行方案：

### 方案1: Cookies导入方案 ⭐⭐⭐⭐⭐

**实现难度**: ⭐⭐
**成功率**: ⭐⭐⭐⭐⭐
**用户体验**: ⭐⭐⭐⭐

**实现步骤**：
1. 提示用户在浏览器中访问起点
2. 用户完成验证（如果有）
3. 用户导出cookies（提供导出工具/指南）
4. 应用使用cookies发送请求

**优点**：
- ✅ 成功率极高（利用浏览器已验证的session）
- ✅ 可以访问登录后的内容
- ✅ 实现相对简单

**缺点**：
- ⚠️ 需要用户手动操作
- ⚠️ Cookies会过期（需要定期更新）

**技术实现**：
```rust
// 1. 从配置文件读取cookies
let cookies = load_cookies("qidian_cookies.txt");

// 2. 构建cookie header
let cookie_header = cookies.iter()
    .map(|(k, v)| format!("{}={}", k, v))
    .join("; ");

// 3. 发送请求
let response = client.get(url)
    .header("Cookie", cookie_header)
    .send()
    .await?;
```

**UI设计**：
```
┌────────────────────────────────────────┐
│ 起点网站需要验证                        │
│                                        │
│ 为了绕过反爬虫保护，请：                │
│ 1. [在浏览器中打开起点]  按钮           │
│ 2. 完成验证（如果有）                  │
│ 3. [导入Cookies]  按钮                 │
│                                        │
│ 📘 查看详细教程                        │
└────────────────────────────────────────┘
```

---

### 方案2: Tauri WebView嵌入方案 ⭐⭐⭐⭐

**实现难度**: ⭐⭐⭐⭐
**成功率**: ⭐⭐⭐⭐⭐
**用户体验**: ⭐⭐⭐⭐⭐

**实现思路**：
1. 在Tauri应用中打开WebView
2. 用户在WebView中访问起点并完成验证
3. 通过JavaScript注入提取页面数据
4. 传递数据回Rust后端

**优点**：
- ✅ 完全在真实浏览器环境中
- ✅ 用户体验最好（所见即所得）
- ✅ 可以处理任何验证

**缺点**：
- ⚠️ 实现复杂度较高
- ⚠️ 需要用户在应用内操作

**技术实现**：
```rust
// Tauri命令 - 打开WebView窗口
#[tauri::command]
async fn open_qidian_window(app_handle: AppHandle) -> Result<()> {
    // 创建新窗口显示起点网站
    WindowBuilder::new(
        &app_handle,
        "qidian",
        WindowUrl::External("https://www.qidian.com".parse()?)
    )
    .title("起点中文网")
    .build()?;
    Ok(())
}

// 通过IPC从WebView获取数据
// Frontend在WebView中执行JavaScript，提取数据后发送消息给Backend
```

**UI流程**：
```
用户操作：导入 > 起点 > 输入书名
     ↓
应用打开内嵌WebView显示起点搜索页
     ↓
用户在WebView中完成验证、搜索、选择书籍
     ↓
注入JavaScript提取书籍信息
     ↓
返回主应用，自动填充表单
```

---

### 方案3: 浏览器扩展协作方案 ⭐⭐⭐

**实现难度**: ⭐⭐⭐
**成功率**: ⭐⭐⭐⭐⭐
**用户体验**: ⭐⭐⭐

**实现思路**：
1. 开发一个浏览器扩展（Chrome/Firefox）
2. 用户在浏览器中选择书籍
3. 扩展提取信息并发送给应用
4. 应用接收数据并保存

**优点**：
- ✅ 在真实浏览器环境
- ✅ 可以持续运行
- ✅ 用户操作简单

**缺点**：
- ⚠️ 需要开发和维护扩展
- ⚠️ 用户需要安装扩展
- ⚠️ 跨浏览器兼容性

**技术实现**：
```javascript
// 浏览器扩展content script
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.action === "extractBookInfo") {
    const bookInfo = {
      title: document.querySelector('.book-info h1').innerText,
      author: document.querySelector('.writer').innerText,
      description: document.querySelector('.intro').innerText,
    };

    // 发送到本地应用（通过native messaging）
    chrome.runtime.sendNativeMessage('com.nothingbut.library', bookInfo);
  }
});
```

---

### 方案4: 移动端API逆向 ⭐⭐

**实现难度**: ⭐⭐⭐⭐⭐
**成功率**: ⭐⭐⭐
**用户体验**: ⭐⭐⭐⭐⭐

**实现思路**：
1. 抓包分析起点APP的API
2. 逆向分析签名算法
3. 实现签名和API调用

**优点**：
- ✅ 完全自动化
- ✅ 不需要浏览器

**缺点**：
- ⚠️ 需要逆向工程
- ⚠️ API可能随时改变
- ⚠️ 可能违反服务条款

**分析工具**：
- Charles Proxy
- mitmproxy
- Frida（动态分析）

**风险**：
- 起点可能检测到并封禁
- 签名算法复杂且经常变化
- 法律和道德风险

---

## 🎯 推荐方案排序

根据**实现难度**、**成功率**、**用户体验**综合评估：

### 第一名：方案1 - Cookies导入 ⭐⭐⭐⭐⭐
- 实现简单
- 成功率高
- 用户操作可接受
- **建议首先实现**

### 第二名：方案2 - WebView嵌入 ⭐⭐⭐⭐
- 用户体验最好
- 实现复杂但可行
- **可以作为高级功能**

### 第三名：方案3 - 浏览器扩展 ⭐⭐⭐
- 需要额外维护
- 适合重度用户
- **可选功能**

### 不推荐：方案4 - API逆向 ⭐⭐
- 难度太高
- 风险太大
- **不建议实施**

---

## 📋 实施路线图

### 阶段1：快速MVP（1周）
**目标**：让起点用户能够导入书籍

1. **实现Cookies导入功能**
   - 创建配置文件格式
   - 实现cookies读取和解析
   - 添加cookies到HTTP请求

2. **UI优化**
   - 添加"获取Cookies"教程按钮
   - 提供清晰的步骤说明
   - 错误提示和调试信息

3. **测试验证**
   - 使用真实cookies测试
   - 验证书籍信息提取
   - 处理cookies过期情况

### 阶段2：体验优化（2-3周）
**目标**：优化用户体验

1. **Cookies管理**
   - 图形化cookies导入界面
   - 一键从浏览器导出cookies工具
   - 自动检测cookies有效性

2. **错误处理**
   - 友好的错误提示
   - Cookies过期提醒
   - 更新cookies引导

### 阶段3：高级功能（可选）
**目标**：提供更好的用户体验

1. **WebView集成**
   - 在应用内打开起点
   - 自动提取信息
   - 无需手动导出cookies

2. **浏览器扩展**
   - 开发Chrome扩展
   - 一键导入到应用

---

## 🛠️ 立即可以测试的方案

### 测试Cookies方案

1. **获取cookies**:
```bash
# 在浏览器中访问起点
1. 打开 https://www.qidian.com
2. 完成验证
3. F12 > Application > Cookies
4. 复制所有cookies
```

2. **创建配置文件**:
```bash
# tests/scraper-test/qidian_cookies.txt
_csrfToken=xxxxxxxxxx
newstatisticUUID=xxxxxxxxxx
_ga=xxxxxxxxxx
```

3. **运行测试**:
```bash
cd tests/scraper-test
cargo run --bin test_with_cookies
```

4. **预期结果**:
- 如果cookies有效 → ✅ 成功获取数据
- 如果cookies无效 → ❌ 仍被拦截

---

## 📝 总结

### 核心发现
1. **起点的反爬虫可以绕过** - 使用真实浏览器的cookies
2. **不需要复杂技术** - 简单的cookie管理就能work
3. **用户操作可接受** - 一次性导出cookies即可

### 最佳实践
- ✅ 使用方案1（Cookies导入）作为基础
- ✅ 后续可以添加方案2（WebView）作为增强
- ✅ 为其他网站保留自动抓取能力
- ✅ 实现优雅降级机制

### 下一步
1. 使用真实cookies测试方案1
2. 如果成功，立即实现到主应用
3. 设计友好的UI引导用户
4. 编写详细的使用文档

---

**结论**：通过Cookies导入方案，我们可以成功绕过起点的反爬虫，而且实现简单，用户体验可接受。这是目前最可行的方案！
