# 小说爬虫工具 - 快速参考

## 🚀 快速开始

### 从头开始抓取
```batch
# Windows
start_fresh.bat

# Linux/Mac
./start_fresh.sh
```

### 继续上次进度
```batch
# Windows
continue.bat

# Linux/Mac
./continue.sh
```

---

## 📊 当前状态

- **总书籍数**: 211
- **有封面**: 164
- **最后抓取ID**: 199
- **成功率**: 197/198 (99.5%)

---

## ⚠️ 重要说明

### start_fresh.bat/sh
- ✅ 清空所有数据
- ✅ 从 ID=1 重新开始
- ⚠️  **不可逆操作**，会删除所有数据！

### continue.bat/sh
- ✅ 保留已有数据
- ✅ 从上次停止处继续
- ✅ 安全，可随时中断

---

## 💡 使用建议

### 首次使用
```batch
start_fresh.bat
```

### 日常更新
```batch
continue.bat
```

### 查看统计
```bash
python main.py stats
```

---

## 🛠️ 手动命令

```bash
# 从指定ID开始
python main.py crawl --mode initial --start 100

# 继续上次
python main.py crawl --mode incremental

# 抓取单本
python main.py single --book-id 123

# 查看统计
python main.py stats
```

---

## 📁 数据位置

- **数据库**: `data/youshu.db`
- **封面图片**: `data/covers/`
- **日志文件**: `logs/crawler.log`

---

## 🔄 进度保存

- 自动保存间隔：每100本
- 保存位置：数据库 `crawl_status` 表
- 断点续传：自动从上次停止处继续

---

## 📝 日志查看

```bash
# 查看运行日志
type logs\crawler.log        # Windows
cat logs/crawler.log         # Linux/Mac

# 查看错误日志
type logs\error.log          # Windows
cat logs/error.log           # Linux/Mac
```

---

**更新时间**: 2026-03-15
**版本**: v1.0
