"""
Source site crawlers package.

This package contains crawlers for various novel websites:
- QidianCrawler: 起点中文网
- ZonghengCrawler: 纵横中文网
- YoudubookCrawler: 柚豆书
- XRZWWCrawler: 星人网文
- CDDAOYUECrawler: 次元乐
- More to be added...
"""

from crawlers.source_crawlers.base_source_crawler import BaseSourceCrawler
from crawlers.source_crawlers.qidian_crawler import QidianCrawler
from crawlers.source_crawlers.zongheng_crawler import ZonghengCrawler
from crawlers.source_crawlers.youdubook_crawler import YoudubookCrawler
from crawlers.source_crawlers.xrzww_crawler import XRZWWCrawler
from crawlers.source_crawlers.cddaoyue_crawler import CDDAOYUECrawler

__all__ = [
    'BaseSourceCrawler',
    'QidianCrawler',
    'ZonghengCrawler',
    'YoudubookCrawler',
    'XRZWWCrawler',
    'CDDAOYUECrawler',
]

# Site name to crawler class mapping
CRAWLER_MAP = {
    'qidian': QidianCrawler,
    'zongheng': ZonghengCrawler,
    'youdubook': YoudubookCrawler,
    'xrzww': XRZWWCrawler,
    'cddaoyue': CDDAOYUECrawler,
}


def get_crawler(site_name: str):
    """
    Get crawler instance by site name.

    Args:
        site_name: Name of the site (e.g., 'qidian', 'zongheng', 'youdubook')

    Returns:
        Crawler instance

    Raises:
        ValueError: If site_name is not supported
    """
    crawler_class = CRAWLER_MAP.get(site_name.lower())
    if not crawler_class:
        raise ValueError(f"Unsupported site: {site_name}. Supported sites: {list(CRAWLER_MAP.keys())}")
    return crawler_class()


def supported_sites():
    """Return list of supported site names."""
    return list(CRAWLER_MAP.keys())
