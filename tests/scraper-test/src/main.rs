/// 起点爬虫测试工具
///
/// 用法：
///   cargo run --bin test_http_direct       # 测试直接HTTP请求
///   cargo run --bin test_book_page         # 测试访问书籍详情页
///   cargo run --bin test_headless_chrome   # 测试Headless Chrome
///
/// 测试数据：
///   书名：黎明之剑
///   作者：远瞳
///   Book ID: 1010400217

fn main() {
    println!("起点爬虫测试工具");
    println!("================\n");
    println!("可用的测试方案：");
    println!("  1. cargo run --bin test_http_direct       - 直接HTTP请求搜索API");
    println!("  2. cargo run --bin test_book_page         - HTTP访问书籍详情页");
    println!("  3. cargo run --bin test_headless_chrome   - Headless Chrome访问详情页");
    println!("\n测试数据：");
    println!("  书名：黎明之剑");
    println!("  作者：远瞳");
    println!("  Book ID: 1010400217");
    println!("\n请使用上面的命令运行具体测试。");
}
