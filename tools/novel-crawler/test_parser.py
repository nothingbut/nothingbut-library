"""
Quick test for parsing youshu page
"""
from bs4 import BeautifulSoup

# Read the saved HTML
with open('test_page.html', 'r', encoding='utf-8') as f:
    html = f.read()

soup = BeautifulSoup(html, 'html.parser')

print("=== Testing Selectors ===\n")

# Test title
title_elem = soup.select_one('span[style*="font-size:20px"]')
print(f"Title selector found: {title_elem is not None}")
if title_elem:
    print(f"Title text: {title_elem.get_text(strip=True)}")

# Test author
author_link = soup.select_one('a[href*="/modules/article/authorarticle.php?author="]')
print(f"\nAuthor selector found: {author_link is not None}")
if author_link:
    print(f"Author text: {author_link.get_text(strip=True)}")

# Test cover
cover_img = soup.select_one('.book-detail-img img')
print(f"\nCover selector found: {cover_img is not None}")
if cover_img:
    print(f"Cover URL: {cover_img.get('src')}")

# Test description
desc_elem = soup.select_one('.tabvalue')
print(f"\nDescription selector found: {desc_elem is not None}")
if desc_elem:
    desc_text = desc_elem.get_text(strip=True)
    print(f"Description (first 100 chars): {desc_text[:100]}")

# Test tags
tag_links = soup.select('.tag-link')
print(f"\nTag selector found: {len(tag_links)} tags")
if tag_links:
    tags = [tag.get_text(strip=True) for tag in tag_links]
    print(f"Tags: {tags}")

# Test status
status_elem = soup.select_one('.author-item-exp')
print(f"\nStatus selector found: {status_elem is not None}")
if status_elem:
    print(f"Status text: {status_elem.get_text(strip=True)}")
