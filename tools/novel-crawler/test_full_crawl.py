"""
Full crawl test with debugging
"""
import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from config import settings
from crawlers.youshu_crawler import YoushuCrawler
from utils.http_client import HTTPClient

print("=== Testing Full Crawl Process ===\n")

# Initialize components
http_client = HTTPClient(settings)
crawler = YoushuCrawler(settings, http_client)

# Test URL building
book_id = 1
url = crawler.build_url(book_id)
print(f"1. URL built: {url}")

# Test fetching
print(f"\n2. Fetching page...")
html = crawler.fetch_page(url)
if html:
    print(f"   [OK] Page fetched successfully")
    print(f"   HTML length: {len(html)} characters")

    # Save to file for debugging
    debug_file = Path(__file__).parent / 'fetched_page.html'
    with open(debug_file, 'w', encoding='utf-8') as f:
        f.write(html)
    print(f"   Saved to: {debug_file}")
else:
    print(f"   [FAIL] Failed to fetch page")
    http_client.close()
    sys.exit(1)

# Test parsing
print(f"\n3. Parsing book info...")
book_info = crawler.parse_book_info(html)
if book_info:
    print(f"   [OK] Parsing successful!")
    print(f"\n   Book Information:")
    for key, value in book_info.items():
        if value:
            print(f"     {key}: {value if len(str(value)) < 100 else str(value)[:100] + '...'}")
else:
    print(f"   [FAIL] Parsing failed")
    print(f"   Checking why...")

    # Debug: try individual selectors
    from bs4 import BeautifulSoup
    soup = BeautifulSoup(html, 'html.parser')

    # Test each selector
    tests = [
        ('Title', 'span[style*="font-size:20px"]'),
        ('Author link', 'a[href*="/modules/article/authorarticle.php?author="]'),
        ('Cover', '.book-detail-img img'),
        ('Description', '.tabvalue'),
        ('Tags', '.tag-link'),
        ('Status', '.author-item-exp'),
    ]

    print(f"\n   Individual selector tests:")
    for name, selector in tests:
        if 'Author link' in name:
            elem = soup.select_one(selector)
        else:
            elem = soup.select_one(selector)
        found = elem is not None
        print(f"     {name}: {'[OK]' if found else '[FAIL]'}")

http_client.close()

print(f"\n=== Test Complete ===")
