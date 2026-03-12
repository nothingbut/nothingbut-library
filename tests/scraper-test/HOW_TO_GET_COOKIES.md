# 如何获取起点网站的Cookies

## 方法1：使用浏览器开发者工具（推荐）

### Chrome浏览器

1. **打开起点网站**
   - 访问：https://www.qidian.com

2. **打开开发者工具**
   - 按 `F12` 键
   - 或者右键点击页面 → 选择"检查"

3. **切换到Application标签**
   - 在开发者工具顶部找到 `Application` 标签
   - 如果看不到，点击 `>>` 按钮查找

4. **展开Cookies**
   - 在左侧栏找到 `Storage` 部分
   - 展开 `Cookies`
   - 点击 `https://www.qidian.com`

5. **复制Cookies**
   - 你会看到一个表格，显示所有cookies
   - 每个cookie有：Name（名称）、Value（值）等列

6. **导出方法A：手动复制**
   ```
   右键点击表格区域 → 选择"全选"
   Ctrl+C 复制
   粘贴到文本文件中
   ```

7. **导出方法B：使用Console**
   - 切换到 `Console` 标签
   - 复制下面的代码并回车：
   ```javascript
   document.cookie.split(';').forEach(c => console.log(c.trim()));
   ```
   - 复制输出的内容

---

### Firefox浏览器

1. **打开起点网站**
   - 访问：https://www.qidian.com

2. **打开开发者工具**
   - 按 `F12` 键
   - 或者右键 → "检查元素"

3. **切换到存储标签**
   - 点击 `存储` 或 `Storage` 标签

4. **查看Cookies**
   - 展开 `Cookie`
   - 点击 `https://www.qidian.com`

5. **复制Cookies**
   - 右键点击任意cookie → "全部复制"
   - 或手动复制每个cookie的名称和值

---

## 方法2：使用自动化脚本（最简单）⭐ 推荐

我们提供了一个JavaScript脚本，可以自动提取cookies。

### 使用步骤

1. **访问起点网站**
   ```
   https://www.qidian.com
   ```

2. **完成任何验证**
   - 如果出现滑动验证码，请完成它
   - 等待页面正常加载

3. **打开Console**
   - 按 `F12` → 切换到 `Console` 标签

4. **粘贴并运行脚本**
   - 复制下面的脚本
   - 粘贴到Console中
   - 按回车运行

```javascript
// 自动提取Cookies脚本
(function() {
  console.log('=== 起点Cookies提取工具 ===\n');

  const cookies = document.cookie.split(';')
    .map(c => c.trim())
    .filter(c => c.length > 0);

  if (cookies.length === 0) {
    console.error('❌ 未找到任何cookies！');
    console.log('请确保：');
    console.log('1. 已经在起点网站上');
    console.log('2. 已完成验证（如果有）');
    return;
  }

  console.log(`找到 ${cookies.length} 个cookies：\n`);

  // 格式化输出（适合直接复制到配置文件）
  console.log('--- 复制下面的内容到 qidian_cookies.txt ---\n');
  cookies.forEach(cookie => {
    console.log(cookie);
  });
  console.log('\n--- 复制结束 ---\n');

  // 也输出为环境变量格式
  const cookieStr = cookies.join('; ');
  console.log('环境变量格式（用于测试）：');
  console.log(`export QIDIAN_COOKIES="${cookieStr}"`);

  console.log('\n✅ 提取完成！请复制上面的内容。');
})();
```

5. **复制输出结果**
   - 在Console中会显示所有cookies
   - 复制 `--- 复制下面的内容 ---` 之间的内容

---

## 方法3：使用环境变量（用于快速测试）

如果你只是想快速测试，可以直接使用环境变量：

```bash
# 在Console中运行方法2的脚本
# 复制最后输出的export命令

# 然后在终端中运行：
export QIDIAN_COOKIES="你的cookies字符串"

# 运行测试
cd tests/scraper-test
cargo run --bin test_with_cookies
```

---

## 保存Cookies到配置文件

### 创建配置文件

```bash
cd tests/scraper-test
cp qidian_cookies.txt.example qidian_cookies.txt
```

### 编辑配置文件

打开 `qidian_cookies.txt`，格式如下：

```
_csrfToken=xxxxxxxxxxxxxx
newstatisticUUID=xxxxxxxxxxxxxx
_ga=GA1.2.xxxxxxxxxx.xxxxxxxxxx
_gid=GA1.2.xxxxxxxxxx.xxxxxxxxxx
```

**每行一个cookie，格式：名称=值**

---

## 常见问题

### Q1: 为什么需要Cookies？
**A**: Cookies包含了你的浏览器验证信息，使用它可以绕过反爬虫检测。

### Q2: Cookies安全吗？
**A**: 请注意：
- ⚠️ 不要分享你的cookies给他人
- ⚠️ Cookies包含登录信息
- ✅ 只保存在本地，不会上传

### Q3: Cookies会过期吗？
**A**: 会的。过期后需要重新获取。通常有效期：
- 几小时到几天不等
- 如果测试失败，可能是cookies过期了

### Q4: 需要所有的Cookies吗？
**A**: 不一定。关键的cookies通常是：
- `_csrfToken` - CSRF令牌
- `newstatisticUUID` - 统计ID
- `_ga`, `_gid` - Google Analytics
- 其他起点特有的cookies

建议：**复制所有cookies**，让程序自动筛选。

### Q5: 我不想手动复制怎么办？
**A**: 等待我们实现以下功能：
1. 浏览器扩展（一键导出）
2. 应用内WebView（自动提取）
3. 图形化导入界面

目前请使用**方法2（自动化脚本）**，最简单！

---

## 快速测试流程

```bash
# 1. 访问起点并提取cookies（使用方法2的脚本）
# 2. 复制输出的export命令

# 3. 在终端运行：
export QIDIAN_COOKIES="你复制的cookies"

# 4. 运行测试
cd tests/scraper-test
cargo run --bin test_with_cookies

# 5. 查看结果
# ✅ 如果成功 → Cookie方案可行！
# ❌ 如果失败 → 可能cookies过期或格式错误
```

---

## 下一步

测试成功后，我们会实现：

1. **图形化界面**
   - 一键导入cookies
   - 自动验证有效性
   - 过期提醒

2. **浏览器扩展**
   - Chrome扩展
   - Firefox插件
   - 一键导出到应用

3. **WebView集成**
   - 应用内浏览器
   - 自动提取
   - 无需手动操作

---

需要帮助？参考视频教程（待创建）或联系支持。
