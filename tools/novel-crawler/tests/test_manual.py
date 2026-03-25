"""
Manual test script for Phase 3 crawler functionality.

This script allows manual testing of the crawler components.
"""
import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from config import settings
from database.db_manager import DatabaseManager
from utils.http_client import HTTPClient
from crawlers.youshu_crawler import YoushuCrawler
from utils.image_downloader import ImageDownloader


def test_database_manager():
    """Test database manager functionality."""
    print("\n" + "=" * 60)
    print("Testing Database Manager")
    print("=" * 60)

    db = DatabaseManager(settings)

    # Test getting statistics
    stats = db.get_statistics()
    print(f"\n📊 Database Statistics:")
    print(f"  Total Books:     {stats.get('total_books', 0)}")
    print(f"  Books with Covers: {stats.get('books_with_covers', 0)}")

    if 'by_source' in stats:
        print(f"\n  Books by Source:")
        for source, count in stats['by_source'].items():
            print(f"    - {source}: {count}")

    if 'last_crawl' in stats:
        last_crawl = stats['last_crawl']
        print(f"\n  Last Crawl:")
        print(f"    Type:    {last_crawl.get('crawl_type')}")
        print(f"    Last ID: {last_crawl.get('last_valid_id')}")
        print(f"    Success: {last_crawl.get('success_count')}")

    print("\n✓ Database Manager test completed!")


def test_http_client():
    """Test HTTP client functionality."""
    print("\n" + "=" * 60)
    print("Testing HTTP Client")
    print("=" * 60)

    client = HTTPClient(settings)

    # Test with a reliable URL
    test_url = "https://httpbin.org/get"

    print(f"\n🌐 Testing HTTP GET to: {test_url}")
    response = client.get(test_url)

    if response:
        print(f"  Status: {response.status_code}")
        print(f"  Content length: {len(response.text)} bytes")
        print("\n✓ HTTP Client test completed!")
    else:
        print("  ✗ Request failed!")
        print("\n⚠ HTTP Client test failed (network may be unavailable)")

    client.close()


def test_youshu_parser():
    """Test YoushuCrawler parser with sample HTML."""
    print("\n" + "=" * 60)
    print("Testing YoushuCrawler Parser")
    print("=" * 60)

    # Sample HTML (this is a template - actual selectors may need adjustment)
    sample_html = """
    <html>
        <body>
            <h1 class="book-title">测试书籍标题</h1>
            <div class="author">测试作者</div>
            <div class="book-description">
                这是一本关于测试的书籍描述内容。
            </div>
            <div class="tag-list">
                <span class="tag">玄幻</span>
                <span class="tag">热血</span>
                <span class="tag">冒险</span>
            </div>
            <div class="book-cover">
                <img src="https://example.com/cover.jpg" alt="封面" />
            </div>
            <div class="book-status">连载中</div>
            <div class="source-site">起点中文网</div>
            <a class="source-link" href="https://book.qidian.com/info/123">查看原文</a>
        </body>
    </html>
    """

    client = HTTPClient(settings)
    crawler = YoushuCrawler(settings, client)

    print("\n📝 Parsing sample HTML...")
    book_info = crawler.parse_book_info(sample_html)

    if book_info:
        print("\n✓ Parse successful!")
        print(f"  Title:       {book_info.get('title')}")
        print(f"  Author:      {book_info.get('author')}")
        print(f"  Description: {book_info.get('description', 'N/A')[:50]}...")
        print(f"  Tags:        {book_info.get('tags')}")
        print(f"  Cover URL:   {book_info.get('cover_url', 'N/A')}")
        print(f"  Source Site: {book_info.get('source_site', 'N/A')}")
        print(f"  Status:      {book_info.get('update_status', 'N/A')}")
    else:
        print("\n✗ Parse failed!")
        print("  Note: This may be due to CSS selector mismatches")
        print("  The actual website structure may differ from the template")

    client.close()


def test_image_downloader():
    """Test image download functionality."""
    print("\n" + "=" * 60)
    print("Testing Image Downloader")
    print("=" * 60)

    downloader = ImageDownloader(settings)

    # Test with a sample image URL
    test_url = "https://httpbin.org/image/png"
    test_book_id = 99999  # Use a high number to avoid conflicts

    print(f"\n🖼️  Testing image download from: {test_url}")
    print(f"   Book ID: {test_book_id}")

    filepath = downloader.download_cover(test_url, test_book_id)

    if filepath:
        print(f"  ✓ Image saved to: {filepath}")
        from pathlib import Path
        if Path(filepath).exists():
            size = Path(filepath).stat().st_size
            print(f"  File size: {size} bytes")
        print("\n✓ Image Downloader test completed!")
    else:
        print("  ✗ Image download failed!")
        print("\n⚠ Image Downloader test failed (network may be unavailable)")


def test_database_operations():
    """Test database CRUD operations."""
    print("\n" + "=" * 60)
    print("Testing Database Operations")
    print("=" * 60)

    db = DatabaseManager(settings)

    # Create a test book
    test_book = {
        'id': 99998,
        'title': 'Test Book for Manual Testing',
        'author': 'Test Author',
        'description': 'This is a test book created during manual testing.',
        'tags': '["test", "manual"]',
        'update_status': '已完成'
    }

    print(f"\n💾 Saving test book: {test_book['title']}")
    result = db.save_book(test_book)

    if result:
        print("  ✓ Book saved successfully!")

        # Try to retrieve it
        print(f"\n📖 Retrieving book ID {test_book['id']}")
        retrieved = db.get_book(test_book['id'])

        if retrieved:
            print(f"  ✓ Book retrieved: {retrieved['title']}")
        else:
            print("  ✗ Failed to retrieve book!")

        # Test search
        print(f"\n🔍 Searching for 'Test'")
        results = db.search_books('Test')
        print(f"  Found {len(results)} results")

    else:
        print("  ✗ Failed to save book!")

    print("\n✓ Database Operations test completed!")


def print_menu():
    """Print test menu."""
    print("\n" + "=" * 60)
    print("Manual Test Suite - Phase 3")
    print("=" * 60)
    print("\nAvailable tests:")
    print("  1. Database Manager")
    print("  2. HTTP Client")
    print("  3. YoushuCrawler Parser")
    print("  4. Image Downloader")
    print("  5. Database Operations")
    print("  6. Run All Tests")
    print("  0. Exit")
    print("\nEnter test number (or 'all' to run all):")


def run_all_tests():
    """Run all manual tests."""
    tests = [
        ("Database Manager", test_database_manager),
        ("HTTP Client", test_http_client),
        ("YoushuCrawler Parser", test_youshu_parser),
        ("Image Downloader", test_image_downloader),
        ("Database Operations", test_database_operations),
    ]

    results = []

    for name, test_func in tests:
        try:
            test_func()
            results.append((name, "PASSED"))
        except Exception as e:
            print(f"\n✗ Test failed with error: {e}")
            results.append((name, "FAILED"))

    # Print summary
    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)

    for name, status in results:
        status_icon = "✓" if status == "PASSED" else "✗"
        print(f"{status_icon} {name}: {status}")

    passed = sum(1 for _, s in results if s == "PASSED")
    total = len(results)
    print(f"\nTotal: {passed}/{total} tests passed")


def main():
    """Main entry point."""
    if len(sys.argv) > 1:
        choice = sys.argv[1]
    else:
        print_menu()
        choice = input("\n> ").strip()

    if choice == '0' or choice.lower() == 'exit':
        print("\n👋 Goodbye!")
        return

    if choice.lower() == 'all':
        run_all_tests()
    elif choice == '1':
        test_database_manager()
    elif choice == '2':
        test_http_client()
    elif choice == '3':
        test_youshu_parser()
    elif choice == '4':
        test_image_downloader()
    elif choice == '5':
        test_database_operations()
    else:
        print(f"\n⚠ Unknown choice: {choice}")
        print("  Please enter a number from 0-6")


if __name__ == '__main__':
    main()
