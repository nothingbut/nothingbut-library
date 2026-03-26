# Phase 3 测试指南

**创建日期**: 2026-03-15
**状态**: 测试就绪

---

## 测试环境准备

### Windows 用户

```bash
cd tools/novel-crawler
setup_and_test.bat
```

该脚本会自动:
1. 创建虚拟环境(如果不存在)
2. 激活虚拟环境
3. 安装所有依赖包
4. 运行测试套件

### macOS/Linux 用户

```bash
cd tools/novel-crawler
chmod +x setup_and_test.sh
./setup_and_test.sh
```

### 手动安装

如果自动脚本失败,可以手动执行:

```bash
# 1. 创建虚拟环境
python -m venv venv

# 2. 激活虚拟环境
# Windows
venv\Scripts\activate
# macOS/Linux
source venv/bin/activate

# 3. 安装依赖
pip install -r requirements.txt

# 4. 运行测试
python tests/test_manual.py all
```

---

## 测试套件

### 1. 手动测试 (test_manual.py)

交互式测试脚本,用于验证各个组件:

```bash
# 运行所有测试
python tests/test_manual.py all

# 交互式菜单
python tests/test_manual.py
```

**测试组件**:
- ✅ Database Manager - 数据库操作
- ✅ HTTP Client - 网络请求
- ✅ YoushuCrawler Parser - HTML解析
- ✅ Image Downloader - 图片下载
- ✅ Database Operations - CRUD操作

### 2. 集成测试 (test_integration.py)

完整的组件集成测试:

```bash
# 运行集成测试
pytest tests/test_integration.py -v

# 或使用Python直接运行
python tests/test_integration.py
```

**测试场景**:
- ✅ 完整爬取流程 (HTTP → 解析 → 数据库)
- ✅ 批量爬取
- ✅ 图片下载
- ✅ 数据库统计
- ✅ 爬取状态追踪
- ✅ 搜索功能

### 3. 单元测试 (所有测试)

运行所有单元测试:

```bash
# 运行所有测试
pytest tests/ -v

# 运行特定测试文件
pytest tests/test_utils.py -v
pytest tests/test_database.py -v
pytest tests/test_crawlers.py -v

# 查看测试覆盖率
pytest tests/ --cov=. --cov-report=term-missing
```

---

## 测试用例清单

### Phase 1-3 总计: **34个测试用例**

| 模块 | 文件 | 测试数 | 状态 |
|------|------|--------|------|
| 工具类 | test_utils.py | 7 | ✅ |
| 数据库 | test_database.py | 13 | ✅ |
| 爬虫 | test_crawlers.py | 11 | ✅ |
| 集成 | test_integration.py | 6 | ✅ |
| 手动 | test_manual.py | 5 | ✅ |

---

## 功能测试

### 测试单本爬取

```bash
# 激活虚拟环境后
python main.py single --book-id 1

# 预期输出:
# ✓ Book 1 saved: [书籍名称]
# Cover downloaded for book 1
```

### 测试数据库统计

```bash
python main.py stats

# 预期输出:
# 📊 Database Statistics:
#   Total Books:     [数量]
#   Books with Covers: [数量]
```

### 测试范围爬取 (小范围)

```bash
# 爬取ID 1-5
python main.py crawl --mode initial --start 1
# 然后按 Ctrl+C 停止(测试用)

# 或修改配置设置更小的停止阈值
# 编辑 config/settings.py
# MAX_CONSECUTIVE_FAILURES = 3  # 改为3次
```

---

## 常见问题

### Q: ModuleNotFoundError: No module named 'requests'

**A**: 虚拟环境未激活或依赖未安装

```bash
# 激活虚拟环境
# Windows
venv\Scripts\activate
# macOS/Linux
source venv/bin/activate

# 安装依赖
pip install -r requirements.txt
```

### Q: 网络请求失败

**A**: 这是正常的,测试脚本使用模拟数据,不需要真实网络

- `test_integration.py` - 使用Mock,不需要网络
- `test_manual.py` - HTTP Client测试需要网络,其他测试使用模拟数据

### Q: CSS选择器不匹配

**A**: 优书网的实际HTML结构可能与示例不同

解决方案:
1. 访问 `https://www.youshu.me/book/1`
2. 使用浏览器开发者工具检查HTML结构
3. 更新 `crawlers/youshu_crawler.py` 中的CSS选择器

---

## 测试报告模板

测试完成后,可以使用以下模板记录结果:

```
## 测试报告 - [日期]

### 环境信息
- 操作系统: [Windows/macOS/Linux]
- Python版本: [3.x.x]
- 测试时间: [时间戳]

### 测试结果
- 手动测试: [通过/失败] - [详情]
- 集成测试: [通过/失败] - [详情]
- 单元测试: [通过率] - [通过数]/[总数]

### 发现的问题
1. [问题描述]
   - 组件: [模块名]
   - 严重性: [高/中/低]
   - 状态: [已修复/待修复]

### 备注
[其他观察或建议]
```

---

## 下一步

测试完成后:

1. **如果测试通过**: 可以继续开发 Phase 4
2. **如果有失败**: 根据错误信息修复代码
3. **如果需要调整CSS选择器**: 访问优书网查看实际HTML结构

---

**测试就绪**: ✅ Phase 3 所有组件已实现并可测试

**建议测试顺序**:
1. 运行 `setup_and_test.bat/.sh` 自动安装和测试
2. 检查测试输出,确认所有组件正常
3. 尝试手动爬取单本书籍验证功能
