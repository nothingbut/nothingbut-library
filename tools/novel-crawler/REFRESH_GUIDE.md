# Refresh 脚本使用说明

## 功能概述

`refresh` 脚本用于**刷新书库**，从 ID 1 开始遍历所有可能的书籍 ID：
- ✅ **跳过已存在**：检查数据库中已有的 ID，直接跳过
- 🆕 **抓取缺失**：对不存在的 ID 尝试抓取
- 🛑 **智能退出**：连续失败 50 次后自动停止（与其他脚本相同的退出条件）

## 使用方法

### 方式一：使用便捷脚本（推荐）

#### Linux/Mac 用户
```bash
cd tools/novel-crawler

# 从 ID 1 开始，无限制刷新
./refresh.sh

# 从 ID 100 开始
./refresh.sh --start 100

# 刷新 ID 1-1000 的范围
./refresh.sh --start 1 --end 1000
```

#### Windows 用户
```batch
cd tools\novel-crawler

# 从 ID 1 开始，无限制刷新
refresh.bat

# 从 ID 100 开始
refresh.bat --start 100

# 刷新 ID 1-1000 的范围
refresh.bat --start 1 --end 1000
```

### 方式二：直接使用 Python 命令

```bash
cd tools/novel-crawler

# 从 ID 1 开始
python main.py refresh --start 1

# 指定范围
python main.py refresh --start 1 --end 1000

# 只刷新特定 ID 之后
python main.py refresh --start 500
```

## 工作流程

1. **检查当前状态**：显示数据库中已有的书籍统计
2. **开始遍历**：从指定的起始 ID 开始
3. **跳过已存在**：对每个 ID，先检查数据库
   - 如果存在 → 跳过，记录为 "Skipped"
   - 如果不存在 → 尝试抓取
4. **抓取缺失**：对不存在的 ID 调用爬虫
5. **智能退出**：连续失败 50 次后停止

## 输出示例

```
========================================
小说爬虫工具 - 刷新脚本
========================================

[1/2] 检查当前状态...

============================================================
  Novel Metadata Crawler v1.0
  小说元数据爬虫系统
============================================================

Database Statistics:
  Total Books:     150
  Books with Covers: 120
  ...

[2/2] 开始刷新抓取（跳过已存在的ID）...
参数：
  起始 ID: 1
  结束 ID: 无限制

============================================================
Starting refresh crawl
Starting from ID 1
Skip existing: True
============================================================

Progress: ID 1 - ✓ Already exists (skipped) (Skipped: 1, Success: 0, Failed: 0)
Progress: ID 2 - ✓ Already exists (skipped) (Skipped: 2, Success: 0, Failed: 0)
...
Progress: ID 151 - Attempting to fetch (Skipped: 150, Success: 0, Failed: 0)
✓ Book 151 saved: 新书标题
Progress: ID 152 - Attempting to fetch (Skipped: 150, Success: 1, Failed: 0)
...

============================================================
REFRESH CRAWL SUMMARY
============================================================
Start ID:        1
End ID:          200
Total Checked:   200
Skipped (exist): 150
Success:         45
Failed:          5
Success Rate:    90.00% (of attempted)
Duration:        300s (5.0m)
============================================================

========================================
✓ 刷新完成！
========================================
```

## 使用场景

### 场景 1：初始填充缺失数据
```bash
./refresh.sh
```
从 ID 1 开始，填补所有缺失的书籍。

### 场景 2：更新特定范围
```bash
./refresh.sh --start 1 --end 5000
```
只检查前 5000 个 ID，快速测试。

### 场景 3：从某个 ID 继续
```bash
./refresh.sh --start 10000
```
假设之前刷新到 9999 时中断，从 10000 继续。

## 与其他脚本的对比

| 脚本 | 功能 | 适用场景 |
|------|------|----------|
| `start_fresh.sh` | 清空数据库，从头抓取 | 全新开始 |
| `continue.sh` | 从最后抓取的 ID 继续 | 增量抓取 |
| **`refresh.sh`** | **遍历所有 ID，跳过已存在** | **填补缺失数据** |

## 注意事项

1. **性能考虑**：refresh 会检查每一个 ID，比增量抓取慢
2. **退出条件**：连续失败 50 次会自动停止，避免无限循环
3. **数据安全**：不会删除或修改已存在的数据
4. **建议使用场景**：
   - ✅ 数据库有空洞（某些 ID 缺失）
   - ✅ 需要确保小 ID 范围完整性
   - ❌ 不适合快速增量更新（应使用 `continue.sh`）

## 参数说明

- `--start ID`：起始书籍 ID（默认：1）
- `--end ID`：结束书籍 ID（可选）
  - 不指定：无限制，直到连续失败 50 次
  - 指定：只检查指定范围

## 故障排除

### Q: 为什么要用 refresh 而不是 continue？
A: `continue` 只从最后一个 ID 继续，不会填补中间的空洞。如果数据库有缺失的 ID（例如 ID 5-10 缺失），refresh 会发现并填补这些缺失。

### Q: refresh 会重复抓取已有的书吗？
A: 不会。refresh 会先检查数据库，如果 ID 已存在就直接跳过，不会重复抓取。

### Q: 如何查看当前数据库状态？
A: 运行 `python main.py stats` 或脚本会自动显示。

### Q: 连续失败 50 次后停止，可以调整吗？
A: 可以修改配置文件中的 `MAX_CONSECUTIVE_FAILURES` 值。
