# 扩展网站爬虫规格说明

**创建日期**: 2026-03-15
**Phase**: Phase 4 - 扩展
**状态**: 规格定义中

---

## 📋 目标网站列表

### 优先级 1 (主流大站)

| 网站名称 | 网址 | 简称 | 优先级 | 预计难度 |
|---------|------|------|--------|----------|
| 刺猬猫 | https://www.ciweimao.com/ | ciweimao | 🔴 高 | 中 |
| 晋江文学 | https://www.jjwxc.net/ | jjwxc | 🔴 高 | 中 |
| 豆瓣阅读 | https://read.douban.com/ | douban | 🔴 高 | 高 |
| 塔读文学 | https://www.tadu.com/ | tadu | 🔴 高 | 中 |
| 17K专题 | https://zhuanti.17k.com/ | 17k | 🔴 高 | 中 |

### 优先级 2 (特色网站)

| 网站名称 | 网址 | 简称 | 优先级 | 预计难度 |
|---------|------|------|--------|----------|
| 次元乐 | https://www.cddaoyue.cn | cddaoyue | 🟡 中 | 低 |
| 柚豆书 | https://www.youdubook.com/ | youdubook | 🟡 中 | 低 |
| 少年梦 | https://www.shaoniandream.com | shaoniandream | 🟡 中 | 中 |
| 星人网文 | https://www.xrzww.com/ | xrzww | 🟡 中 | 低 |

---

## 🔍 网站分析

### 1. 刺猬猫 (ciweimao)

**网站特点**:
- 主要面向二次元用户
- 需要登录才能查看部分内容
- 可能有反爬虫机制

**数据字段**:
- 书名、作者、简介
- 分类（二次元、轻小说等）
- 字数、章节数
- 状态（连载/完结）
- 收藏数、推荐数
- 封面图片

**URL 模式**:
```
书籍详情页: https://www.ciweimao.com/book/{book_id}
```

**POC 策略**:
1. 分析页面 HTML 结构
2. 识别主要 CSS 选择器
3. 测试反爬虫机制
4. 实现基础爬虫

---

### 2. 晋江文学 (jjwxc)

**网站特点**:
- 女性向文学网站
- 大量原创作品
- 可能需要 Cookie

**数据字段**:
- 书名、作者、简介
- 分类标签
- 收藏数、评论数
- 积分/营养值
- 状态、字数
- 封面图片

**URL 模式**:
```
书籍详情页: https://www.jjwxc.net/onebook.php?novelid={book_id}
```

**POC 策略**:
1. 分析页面结构
2. 测试是否需要登录
3. 提取主要数据字段
4. 处理特殊编码

---

### 3. 豆瓣阅读 (douban)

**网站特点**:
- 高质量出版物
- 评分系统完善
- 可能需要 API Token

**数据字段**:
- 书名、作者、出版社
- ISBN、出版日期
- 评分、评价数
- 简介、目录
- 价格、页数
- 封面图片

**URL 模式**:
```
书籍详情页: https://read.douban.com/ebook/{book_id}
```

**POC 策略**:
1. 检查是否需要 API
2. 分析 HTML 结构
3. 提取评分信息
4. 处理出版相关字段

---

### 4. 塔读文学 (tadu)

**网站特点**:
- 网文阅读平台
- VIP 章节
- 移动端适配

**数据字段**:
- 书名、作者、简介
- 分类、标签
- 字数、更新状态
- 推荐票、收藏
- 封面图片

**URL 模式**:
```
书籍详情页: https://www.tadu.com/book/{book_id}
```

**POC 策略**:
1. 分析 PC/移动端页面
2. 识别 VIP 标识
3. 提取基础数据
4. 测试稳定性

---

### 5. 17K专题 (17k)

**网站特点**:
- 17K 旗下专题页
- 精选作品合集
- 可能需要特殊处理

**数据字段**:
- 专题名称、描述
- 包含书籍列表
- 每本书的基本信息
- 封面图片

**URL 模式**:
```
专题页: https://zhuanti.17k.com/{topic_id}
```

**POC 策略**:
1. 分析专题结构
2. 提取书籍列表
3. 获取每本书详情
4. 处理分页

---

### 6. 次元乐 (cddaoyue)

**网站特点**:
- 动漫相关
- 轻小说为主
- 结构相对简单

**数据字段**:
- 书名、作者
- 简介、标签
- 分类
- 封面图片

**URL 模式**:
```
书籍详情页: https://www.cddaoyue.cn/book/{book_id}
```

**POC 策略**:
1. 简单的 HTML 解析
2. 基础字段提取
3. 测试稳定性

---

### 7. 柚豆书 (youdubook)

**网站特点**:
- 中型平台
- 较少反爬限制
- 适合 POC

**数据字段**:
- 书名、作者
- 简介、分类
- 字数、状态
- 封面图片

**URL 模式**:
```
书籍详情页: https://www.youdubook.com/book/{book_id}
```

**POC 策略**:
1. 最简单的实现
2. 验证基础功能
3. 作为模板示例

---

### 8. 少年梦 (shaoniandream)

**网站特点**:
- 青少年文学
- 可能有特殊分类
- 结构适中

**数据字段**:
- 书名、作者
- 简介、分类
- 适读年龄
- 封面图片

**URL 模式**:
```
书籍详情页: https://www.shaoniandream.com/book/{book_id}
```

**POC 策略**:
1. 分析页面结构
2. 提取特色字段
3. 测试稳定性

---

### 9. 星人网文 (xrzww)

**网站特点**:
- 小型平台
- 结构简单
- 适合快速实现

**数据字段**:
- 书名、作者
- 简介、分类
- 字数、状态
- 封面图片

**URL 模式**:
```
书籍详情页: https://www.xrzww.com/book/{book_id}
```

**POC 策略**:
1. 快速实现
2. 验证框架
3. 作为示例

---

## 📊 实施计划

### 批次 1: POC 验证 (第1周)

**目标**: 验证可行性，建立模板

选择网站：
1. ✅ 柚豆书 (youdubook) - 结构最简单
2. ✅ 星人网文 (xrzww) - 小型平台
3. ✅ 次元乐 (cddaoyue) - 中型平台

**任务**:
- 创建爬虫基类模板
- 实现 3 个 POC 爬虫
- 编写单元测试
- 验证数据提取
- 建立最佳实践

**交付物**:
- 3 个完整爬虫
- 测试用例
- 实现指南

---

### 批次 2: 主流网站 (第2-3周)

**目标**: 实现重要主流网站

选择网站：
1. ✅ 刺猬猫 (ciweimao) - 二次元平台
2. ✅ 晋江文学 (jjwxc) - 女频平台
3. ✅ 塔读文学 (tadu) - 网文平台

**任务**:
- 研究反爬虫机制
- 实现登录/认证
- 处理特殊编码
- 优化性能
- 完善错误处理

**交付物**:
- 3 个完整爬虫
- 集成测试
- 性能报告

---

### 批次 3: 高级网站 (第4周)

**目标**: 实现复杂网站

选择网站：
1. ✅ 豆瓣阅读 (douban) - 需要特殊处理
2. ✅ 17K专题 (17k) - 专题页面
3. ✅ 少年梦 (shaoniandream) - 特色字段

**任务**:
- 处理复杂页面结构
- 实现 API 调用（如需要）
- 提取复杂字段
- 完善数据标准化

**交付物**:
- 3 个完整爬虫
- 完整文档
- 使用示例

---

## 🔧 技术架构

### 爬虫基类

```python
class BaseSourceCrawler(ABC):
    """所有来源爬虫的基类"""

    @abstractmethod
    def get_site_name(self) -> str:
        """返回站点名称"""

    @abstractmethod
    def get_base_url(self) -> str:
        """返回基础URL模式"""

    @abstractmethod
    def build_book_url(self, book_id: str) -> str:
        """构建书籍URL"""

    @abstractmethod
    def parse_book_detail(self, html: str, book_id: str) -> Optional[Dict]:
        """解析书籍详情"""

    def crawl_book_detail(self, book_id: str) -> Optional[Dict]:
        """完整爬取流程"""
```

### 数据模型

所有爬虫返回统一的数据格式：

```python
{
    'book_id': str,           # 书籍ID
    'title': str,             # 书名
    'author': str,            # 作者
    'description': str,       # 简介
    'category': str,          # 主分类
    'sub_category': str,      # 子分类
    'tags': List[str],        # 标签列表
    'cover_url': str,         # 封面URL
    'cover_path': str,        # 本地封面路径
    'word_count': int,        # 字数
    'chapter_count': int,     # 章节数
    'status': str,            # 状态
    'rating': float,          # 评分
    'view_count': int,        # 浏览量
    'favorite_count': int,    # 收藏量
}
```

---

## 📝 POC 检查清单

每个网站的 POC 需要验证：

### 基础功能
- [ ] 能否访问页面
- [ ] HTML 能否正确解析
- [ ] 书名能否提取
- [ ] 作者能否提取
- [ ] 简介能否提取
- [ ] 封面URL能否提取

### 高级功能
- [ ] 分类能否正确识别
- [ ] 标签能否提取
- [ ] 字数/章节数能否获取
- [ ] 评分能否提取
- [ ] 状态能否判断

### 稳定性
- [ ] 错误处理是否完善
- [ ] 重试机制是否有效
- [ ] 日志是否清晰
- [ ] 性能是否可接受

### 兼容性
- [ ] 数据格式是否统一
- [ ] 字段映射是否正确
- [ ] 数据库能否正确存储

---

## 🚀 快速开始

### POC 实现步骤

1. **创建爬虫类**
```python
# crawlers/source_crawlers/newsite_crawler.py
from crawlers.source_crawlers.base_source_crawler import BaseSourceCrawler

class NewSiteCrawler(BaseSourceCrawler):
    def get_site_name(self):
        return 'newsite'

    def get_base_url(self):
        return "https://www.newsite.com/book/{book_id}"

    def build_book_url(self, book_id):
        return f'https://www.newsite.com/book/{book_id}'

    def parse_book_detail(self, html, book_id):
        # 实现解析逻辑
        pass
```

2. **注册爬虫**
```python
# crawlers/source_crawlers/__init__.py
CRAWLER_MAP = {
    'newsite': NewSiteCrawler,
}
```

3. **编写测试**
```python
# tests/test_newsite_crawler.py
class TestNewSiteCrawler:
    def test_parse_book_detail(self):
        crawler = NewSiteCrawler()
        # 测试代码
```

4. **验证功能**
```bash
pytest tests/test_newsite_crawler.py -v
python main.py source --mode batch --ids 1
```

---

## 📚 参考资源

### 已实现的爬虫
- QidianCrawler: 起点中文网
- ZonghengCrawler: 纵横中文网

### 参考文档
- Phase 4 Summary: `docs/PHASE4_SUMMARY.md`
- Source Crawler Guide: `SOURCE_CRAWLER_GUIDE.md`

---

## 🎯 成功标准

### 批次 1 (POC)
- ✅ 3个网站爬虫实现
- ✅ 基础数据提取正常
- ✅ 单元测试通过
- ✅ 文档完善

### 批次 2 (主流)
- ✅ 3个主流网站实现
- ✅ 反爬虫处理
- ✅ 性能优化
- ✅ 集成测试通过

### 批次 3 (高级)
- ✅ 3个复杂网站实现
- ✅ 特殊功能支持
- ✅ 完整文档
- ✅ 生产就绪

---

**文档版本**: v1.0
**创建日期**: 2026-03-15
**状态**: 准备开始批次 1
