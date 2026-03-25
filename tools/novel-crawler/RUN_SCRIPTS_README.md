# 小说爬虫运行脚本使用说明

本目录包含两个便捷脚本来运行小说爬虫工具。

## 脚本说明

### 1. 从头开始抓取（start_fresh）

**功能**：
- 清空数据库中的所有书籍记录
- 删除所有已下载的封面图片
- 从 ID=1 开始重新抓取数据

**适用场景**：
- 首次运行爬虫
- 数据库损坏需要重建
- 需要完全重新抓取所有数据

**使用方法**：

#### Windows
```batch
start_fresh.bat
```

#### Linux/Mac
```bash
chmod +x start_fresh.sh
./start_fresh.sh
```

**安全提示**：
脚本会显示警告信息并等待用户确认，确保不会意外删除数据。

---

### 2. 继续上次进度（continue）

**功能**：
- 显示当前数据库统计信息
- 从上次停止的位置继续抓取
- 自动保存进度，每100本书记录一次

**适用场景**：
- 爬虫意外停止后继续
- 定期增量更新
- 添加新书籍

**使用方法**：

#### Windows
```batch
continue.bat
```

#### Linux/Mac
```bash
chmod +x continue.sh
./continue.sh
```

---

## 执行流程

### start_fresh 执行流程

```
[1/3] 清空数据库和封面
  ↓
[2/3] 从 ID=1 开始抓取数据
  ↓ (每100本书自动保存进度)
[3/3] 显示统计信息
  ↓
完成
```

### continue 执行流程

```
[1/2] 检查当前状态
  ↓
[2/2] 继续抓取（增量模式）
  ↓ (每100本书自动保存进度)
完成
```

---

## 工作原理

### 进度保存机制

爬虫每抓取100本书会自动保存一次进度到数据库的 `crawl_status` 表中：

```python
crawl_status 表字段：
- last_valid_id: 最后成功的书籍ID
- success_count: 成功数量
- failure_count: 失败数量
- failed_ids: 失败的ID列表
- crawl_type: 抓取类型（initial/incremental）
```

### 断点续传原理

增量模式会：
1. 从数据库读取 `last_valid_id`
2. 从 `last_valid_id + 1` 开始继续抓取
3. 遇到连续50次失败自动停止（避免无限循环）

### 智能停止机制

爬虫会在以下情况自动停止：
- 连续失败达到 `MAX_CONSECUTIVE_FAILURES`（默认50次）
- 到达指定的结束ID（如果设置了 `--end` 参数）
- 手动中断（Ctrl+C）

---

## 注意事项

### 数据安全

⚠️ **重要提示**：
1. `start_fresh.bat/sh` 会**永久删除**所有数据，请谨慎使用
2. 建议在清空前备份数据库文件：`data/youshu.db`
3. 封面图片文件位于 `data/covers/` 目录

### 性能优化

1. **抓取速度**：1000-2000 本/小时
2. **内存使用**：约 50-100MB
3. **网络延迟**：每次请求间隔 1-3 秒（避免被封）
4. **失败重试**：自动重试 3 次

### 错误处理

如果遇到错误：
1. 查看日志文件：`logs/crawler.log`
2. 检查错误日志：`logs/error.log`
3. 使用 `python main.py stats` 查看当前状态
4. 如果需要，可以重新运行 `continue.bat/sh`

---

## 高级用法

### 手动命令

除了使用脚本，也可以直接运行 Python 命令：

```bash
# 从头开始（不删除数据）
python main.py crawl --mode initial --start 1

# 继续上次进度
python main.py crawl --mode incremental

# 指定起始ID
python main.py crawl --mode initial --start 1000

# 抓取单本书
python main.py single --book-id 123

# 查看统计
python main.py stats
```

### 定时任务

在 Linux/Mac 上可以使用 cron 定时运行：

```bash
# 编辑 crontab
crontab -e

# 每天凌晨2点运行增量抓取
0 2 * * * cd /path/to/novel-crawler && ./continue.sh >> logs/cron.log 2>&1
```

在 Windows 上可以使用任务计划程序：
1. 打开"任务计划程序"
2. 创建基本任务
3. 设置触发器（每天特定时间）
4. 操作：启动程序 `continue.bat`

---

## 故障排除

### 问题1：脚本无法运行

**Windows**：
```batch
# 确保使用正确的编码
chcp 65001
start_fresh.bat
```

**Linux/Mac**：
```bash
# 添加执行权限
chmod +x start_fresh.sh continue.sh
```

### 问题2：数据库锁定

如果提示数据库被锁定：
```bash
# 检查是否有其他Python进程在运行
ps aux | grep python

# 或者重启脚本
# 进度已保存，可以直接运行 continue.sh
```

### 问题3：网络连接问题

如果抓取大量失败：
1. 检查网络连接
2. 查看日志文件确认错误类型
3. 可能是目标网站反爬虫策略更新
4. 可以调整 `config/settings.py` 中的延迟时间

---

## 文件说明

```
novel-crawler/
├── start_fresh.bat          # Windows: 从头开始脚本
├── start_fresh.sh           # Linux/Mac: 从头开始脚本
├── continue.bat             # Windows: 继续进度脚本
├── continue.sh              # Linux/Mac: 继续进度脚本
├── main.py                  # 主程序入口
├── data/
│   ├── youshu.db           # 主数据库
│   └── covers/             # 封面图片目录
└── logs/                    # 日志文件目录
```

---

## 技术支持

如有问题，请查看：
1. 项目 README.md
2. 日志文件 `logs/crawler.log` 和 `logs/error.log`
3. 使用 `python main.py stats` 查看数据库状态

**创建日期**: 2026-03-15
**版本**: v1.0
