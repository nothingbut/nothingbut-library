# 任务调度系统 - 使用指南

**Phase 5 完成** - 2026-03-15

---

## 🎉 新功能

现在可以使用定时任务自动爬取和更新数据！

### 调度功能

- ✅ 每日自动增量爬取
- ✅ 失败任务自动重试
- ✅ 定时统计报告
- ✅ 灵活的任务调度

---

## 🚀 快速开始

### 1. 查看计划任务

```bash
cd tools/novel-crawler
python main.py schedule list
```

输出示例：
```
Scheduled Jobs:

  ID: daily_crawl
  Name: Daily Incremental Crawl
  Trigger: cron[hour='2', minute='0']
  Next Run: 2026-03-16T02:00:00+08:00

  ID: retry_failed
  Name: Retry Failed Jobs
  Trigger: cron[day_of_week='0', hour='3', minute='0']
  Next Run: 2026-03-16T03:00:00+08:00

  ID: daily_report
  Name: Daily Statistics Report
  Trigger: cron[hour='6', minute='0']
  Next Run: 2026-03-16T06:00:00+08:00
```

### 2. 启动调度器

```bash
# 启动调度器（持续运行）
python main.py schedule start
```

调度器将持续运行并按计划执行任务。按 `Ctrl+C` 可以优雅停止。

### 3. 立即执行任务

```bash
# 执行每日爬取（包含所有子任务）
python main.py schedule run --job daily_crawl

# 只执行失败任务重试
python main.py schedule run --job retry_failed

# 只生成统计报告
python main.py schedule run --job daily_report

# 执行优书增量爬取
python main.py schedule run --job incremental_youshu

# 执行来源网站爬取（可指定限制）
python main.py schedule run --job source_crawl
```

---

## 📋 计划任务说明

### 1. daily_crawl - 每日增量爬取

**执行时间**: 每天凌晨 2:00

**包含任务**:
- ✅ 增量爬取优书网
- ✅ 爬取来源网站详情（默认100本）
- ✅ 重试失败任务
- ✅ 生成每日报告

**用途**: 主要的每日更新任务，自动化所有爬取流程。

### 2. retry_failed - 失败任务重试

**执行时间**: 每周日凌晨 3:00

**包含任务**:
- ✅ 获取所有失败的书籍ID
- ✅ 重新爬取失败的书籍
- ✅ 更新数据库状态

**用途**: 定期重试之前失败的任务，提高数据完整性。

### 3. daily_report - 每日统计报告

**执行时间**: 每天早上 6:00

**包含任务**:
- ✅ 生成优书网统计
- ✅ 生成来源网站统计
- ✅ 汇总报告

**用途**: 生成每日数据报告，便于监控和分析。

---

## 📊 每日任务执行流程

```
daily_crawl (2:00 AM)
    │
    ├─► 1. 增量优书爬取
    │       └─► 检测新书籍
    │       └─► 更新已有书籍
    │       └─► 下载封面
    │
    ├─► 2. 来源网站爬取
    │       ├─► 起点详情
    │       ├─► 纵横详情
    │       └─► 保存到独立数据库
    │
    ├─► 3. 失败任务重试
    │       ├─► 获取失败ID
    │       └─► 重新爬取
    │
    └─► 4. 生成报告
            ├─► 优书统计
            ├─► 来源统计
            └─► 汇总报告
```

---

## 🛠️ 高级用法

### 自定义参数执行

```bash
# 爬取来源网站时限制为50本
python main.py schedule run --job source_crawl --kwargs '{"source_limit": 50}'

# 爬取来源网站时限制为200本
python main.py schedule run --job source_crawl --kwargs '{"source_limit": 200}'
```

### Python API 使用

```python
from scheduler.job_manager import JobManager

# 创建调度器
with JobManager() as manager:
    # 立即执行任务
    result = manager.run_job_now('daily_crawl')
    print(result)

    # 获取任务列表
    jobs = manager.get_scheduled_jobs()
    for job in jobs:
        print(f"{job['name']}: {job['next_run_time']}")

    # 查看执行历史
    history = manager.get_job_history(limit=10)
    for record in history:
        print(f"{record['timestamp']}: {record['job_id']} - {record['status']}")
```

---

## ⏰ 调度时间表

| 时间 | 任务 | 描述 |
|------|------|------|
| 每日 02:00 | daily_crawl | 完整的每日爬取流程 |
| 每周日 03:00 | retry_failed | 重试所有失败任务 |
| 每日 06:00 | daily_report | 生成每日统计报告 |

---

## 📝 任务执行日志

调度器会输出详细的执行日志：

```
============================================================
  DAILY TASK EXECUTION
============================================================

Starting incremental youshu crawl
============================================================
Starting crawl for book 15
✓ Successfully crawled book 15
...
Incremental youshu crawl completed:
  Success: 10
  Failed: 2
  Duration: 45.2s

Starting source site crawl (limit=100)
...
Source site crawl completed:
  Total: 50
  Success: 45
  Failed: 5
  By Site:
    qidian:
      Success: 30
      Failed: 2
    zongheng:
      Success: 15
      Failed: 3

Starting failed job retry
...
Retry completed:
  Retried: 5
  Success: 3
  Failed: 2

Generating daily statistics report
...
Daily Statistics Report:
Youshu Database:
  Total Books: 250
  Books with Covers: 240
  Last Crawl:
    Type: incremental
    Last ID: 260
    Success: 10
    Failed: 2

Source Sites:
  QIDIAN:
    Total Books: 100
    Books with Covers: 95
    Average Rating: 8.5
  ZONGHENG:
    Total Books: 80
    Books with Covers: 75
    Average Rating: 8.2

============================================================
  DAILY TASK SUMMARY
============================================================

✓ incremental_youshu_crawl
✓ source_site_crawl
✓ retry_failed_jobs
✓ daily_report

Total: 4/4 tasks completed successfully
```

---

## 🔄 持续运行模式

### 启动调度器

```bash
python main.py schedule start
```

调度器将：
- ✅ 在后台持续运行
- ✅ 按计划自动执行任务
- ✅ 记录执行历史
- ✅ 处理异常和重试
- ✅ 响应 Ctrl+C 优雅关闭

### 停止调度器

按 `Ctrl+C` 即可优雅停止调度器。

停止时会：
- ✅ 等待当前任务完成
- ✅ 关闭所有连接
- ✅ 保存状态

---

## 🚨 错误处理

调度器具有完善的错误处理机制：

### 任务失败处理

- ✅ 捕获异常并记录
- ✅ 保存到执行历史
- ✅ 不影响其他任务
- ✅ 自动重试机制

### 优雅关闭

- ✅ 捕获 SIGINT (Ctrl+C)
- ✅ 捕获 SIGTERM
- ✅ 等待任务完成
- ✅ 清理资源

---

## 📈 监控和维护

### 查看执行历史

```python
from scheduler.job_manager import JobManager

with JobManager() as manager:
    history = manager.get_job_history(limit=20)
    for record in history:
        print(f"{record['timestamp']} - {record['job_id']}: {record['status']}")
```

### 查看调度状态

```bash
python main.py schedule status
```

### 查看任务列表

```bash
python main.py schedule list
```

---

## 🎯 使用场景

### 场景 1: 完全自动化

```bash
# 启动调度器后无需干预
python main.py schedule start
```

所有任务将按计划自动执行。

### 场景 2: 手动触发

```bash
# 需要时手动执行特定任务
python main.py schedule run --job daily_crawl
```

### 场景 3: 混合模式

```bash
# 调度器处理常规任务
python main.py schedule start

# 另一个终端执行额外任务
python main.py schedule run --job source_crawl --kwargs '{"source_limit": 200}'
```

---

## ⚙️ 配置调整

### 修改执行时间

可以在 `scheduler/job_manager.py` 中修改 cron 表达式：

```python
# 将 daily_crawl 改为每天 3:00 AM
self.scheduler.add_job(
    func=self.daily_task.run_all_daily_tasks,
    trigger=CronTrigger.from_crontab('0 3 * * *'),  # 修改这里
    id='daily_crawl',
    name='Daily Incremental Crawl',
    kwargs={'source_limit': 100},
    replace_existing=True
)
```

### 修改默认参数

```python
# 修改默认的来源网站爬取限制
kwargs={'source_limit': 200}  # 从100改为200
```

---

## 🧪 测试

```bash
# 运行调度器测试
pytest tests/test_scheduler.py -v

# 测试单个任务
python main.py schedule run --job daily_report
```

---

## ⚠️ 注意事项

1. **时区设置**: 调度器使用 Asia/Shanghai 时区
2. **系统时间**: 确保系统时间准确
3. **网络稳定**: 调度器需要稳定的网络连接
4. **磁盘空间**: 确保有足够的磁盘空间存储数据
5. **日志监控**: 定期检查日志文件

---

## 📞 故障排查

### 任务未执行

1. 检查调度器是否运行：`python main.py schedule list`
2. 检查日志文件：`logs/crawler.log`
3. 检查系统时间是否正确

### 任务失败

1. 查看日志中的错误信息
2. 检查网络连接
3. 验证数据库状态
4. 手动执行任务测试

### 调度器无法启动

1. 检查端口占用
2. 检查数据库连接
3. 查看错误日志

---

**生成时间**: 2026-03-15
**版本**: Phase 5 完成
**状态**: 可用 ✅
