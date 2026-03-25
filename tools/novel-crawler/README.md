# Novel Metadata Crawler

小说元数据爬虫系统 - 用于自动采集小说元数据并存储到本地数据库。

## 功能特性

- ✅ 自动爬取优书网书籍索引
- ✅ 下载封面图片到本地
- ✅ 支持起点、纵横等来源网站
- ✅ 定时增量更新
- ✅ 失败自动重试
- ✅ 完整的日志记录

## 快速开始

### 1. 安装依赖

```bash
cd tools/novel-crawler
python -m venv venv

# Windows
venv\Scripts\activate

# macOS/Linux
source venv/bin/activate

pip install -r requirements.txt
```

### 2. 配置设置

编辑 `config/settings.py` 修改配置。

### 3. 运行爬虫

```bash
# 首次全量爬取
python main.py crawl --mode initial --start 1

# 增量更新
python main.py crawl --mode incremental

# 爬取单本书籍测试
python main.py crawl --book-id 123

# 启动定时任务
python main.py schedule --start

# 查看统计信息
python main.py stats
```

## 项目结构

```
novel-crawler/
├── config/                 # 配置文件
├── crawlers/               # 爬虫模块
│   ├── base_crawler.py     # 基础爬虫类
│   ├── youshu_crawler.py   # 优书网爬虫
│   └── source_crawlers/    # 来源网站爬虫
├── database/               # 数据库操作
│   ├── db_manager.py       # 数据库管理
│   └── models.py           # 数据模型
├── utils/                  # 工具类
│   ├── http_client.py      # HTTP客户端
│   ├── image_downloader.py # 图片下载
│   ├── retry_handler.py    # 重试处理
│   └── logger.py           # 日志工具
├── scheduler/              # 任务调度
│   ├── daily_task.py       # 每日任务
│   └── job_manager.py      # 任务管理
├── tests/                  # 测试文件
├── data/                   # 数据目录
│   ├── youshu.db          # 主数据库
│   ├── qidian.db          # 起点数据库
│   └── covers/            # 封面图片
├── logs/                   # 日志目录
└── main.py                 # 主程序入口
```

## 技术栈

- **Python 3.8+**
- **HTTP**: requests
- **HTML解析**: BeautifulSoup4 + lxml
- **数据库**: SQLite
- **调度**: APScheduler
- **测试**: pytest

## 合规性

⚠️ **重要提示**:
- 本爬虫仅用于采集元数据,不爬取小说正文
- 请遵守目标网站的 robots.txt 协议
- 控制请求频率,避免对目标网站造成压力
- 仅用于学习和个人使用

## 性能指标

- 爬取速度: 1000-2000 本/小时
- 成功率: > 95%
- 封面下载成功率: > 95%
- 数据完整性: > 98%

## 开发计划

当前开发阶段: **Phase 1: 项目初始化**

- [x] Phase 1: 项目初始化
- [ ] Phase 2: 基础框架开发
- [ ] Phase 3: 优书网爬虫开发
- [ ] Phase 4: 来源网站爬虫开发
- [ ] Phase 5: 任务调度系统
- [ ] Phase 6: 测试和优化
- [ ] Phase 7: 文档和部署

详细开发计划请参考 `docs/DEVELOPMENT_PLAN.md`

## 文档

- [需求文档](docs/crawler/REQUIREMENTS.md)
- [设计文档](docs/crawler/DESIGN.md)
- [开发计划](docs/crawler/DEVELOPMENT_PLAN.md)
- [交接文档](docs/crawler/HANDOFF.md)

## 许可证

MIT License

---

**创建日期**: 2026-03-15
**版本**: v1.0
**状态**: 开发中
