# 起点爬虫测试工具

测试不同的爬虫方案，验证可行性。

## 测试数据

- **书名**：黎明之剑
- **作者**：远瞳
- **Book ID**：1010400217

## 测试方案

### 方案1：直接HTTP请求搜索API

测试直接调用起点的搜索API。

```bash
cd tests/scraper-test
cargo run --bin test_http_direct
```

**预期结果**：
- ❌ 被WAF拦截，返回验证页面

---

### 方案2：HTTP访问书籍详情页

测试直接访问书籍详情页（已知book ID）。

```bash
cd tests/scraper-test
cargo run --bin test_book_page
```

**预期结果**：
- 需要验证是否能获取到完整HTML
- 验证能否提取书名、作者、简介等信息

---

### 方案3：Headless Chrome访问详情页

使用无头浏览器访问书籍详情页，执行JavaScript。

```bash
cd tests/scraper-test
cargo run --bin test_headless_chrome
```

**预期结果**：
- 首次运行会下载Chromium（~100MB）
- 验证是否能绕过WAF
- 验证能否提取书名、作者、简介等信息

---

## 快速运行所有测试

```bash
cd tests/scraper-test

echo "=== 测试方案1 ==="
cargo run --bin test_http_direct
echo ""

echo "=== 测试方案2 ==="
cargo run --bin test_book_page
echo ""

echo "=== 测试方案3 ==="
cargo run --bin test_headless_chrome
```

## 测试结果分析

根据测试结果，我们将选择最可行的方案集成到主应用中。

### 判断标准

1. **能否获取数据**：是否能成功获取书籍信息
2. **稳定性**：是否经常被WAF拦截
3. **性能**：响应速度如何
4. **维护成本**：方案复杂度

## 目录结构

```
tests/scraper-test/
├── Cargo.toml
├── README.md
└── src/
    ├── main.rs                    # 使用说明
    ├── test_http_direct.rs        # 方案1：HTTP直接请求
    ├── test_book_page.rs          # 方案2：HTTP访问详情页
    └── test_headless_chrome.rs    # 方案3：Headless Chrome
```
