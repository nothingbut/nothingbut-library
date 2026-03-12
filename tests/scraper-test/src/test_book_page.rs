/// 测试方案2：直接访问书籍详情页
/// 使用已知的book ID: 1010400217 (黎明之剑)

use reqwest;
use scraper::{Html, Selector};

fn main() {
    println!("=== 测试方案2：直接访问书籍详情页 ===\n");

    let book_id = "1010400217"; // 黎明之剑的book ID
    let book_url = format!("https://www.qidian.com/book/{}/", book_id);

    println!("测试数据：");
    println!("  Book ID: {}", book_id);
    println!("  URL: {}\n", book_url);

    println!("发送HTTP请求...");

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .cookie_store(true)
        .build()
        .expect("Failed to build client");

    match client.get(&book_url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .send()
    {
        Ok(response) => {
            println!("✅ 收到响应");
            println!("状态码: {}", response.status());

            match response.text() {
                Ok(html) => {
                    println!("HTML长度: {} bytes", html.len());
                    println!("HTML前500字符:");
                    println!("{}\n", &html.chars().take(500).collect::<String>());

                    if html.len() < 1000 {
                        println!("⚠️  HTML太短，可能被WAF拦截\n");
                        return;
                    }

                    println!("开始解析HTML...");
                    let document = Html::parse_document(&html);

                    // 尝试多种选择器模式
                    let selectors = vec![
                        ("书名", vec!["h1", ".book-info h1", "meta[property='og:title']"]),
                        ("作者", vec![".writer", ".author-name-text", ".book-info .writer"]),
                        ("封面", vec!["#bookImg img", ".book-img img"]),
                        ("简介", vec![".book-intro p", "#intro p", ".intro"]),
                        ("分类", vec![".tag", "a.red", ".book-info .tag"]),
                    ];

                    for (name, selector_list) in selectors {
                        println!("\n尝试提取【{}】:", name);
                        for selector_str in selector_list {
                            if let Ok(selector) = Selector::parse(selector_str) {
                                if let Some(element) = document.select(&selector).next() {
                                    let text = element.text().collect::<String>().trim().to_string();
                                    if !text.is_empty() {
                                        println!("  ✅ 选择器 '{}': {}", selector_str,
                                            if text.len() > 100 {
                                                format!("{}...", &text[..100])
                                            } else {
                                                text
                                            }
                                        );
                                    } else if name == "封面" {
                                        // 对于图片，尝试获取src属性
                                        if let Some(src) = element.value().attr("src") {
                                            println!("  ✅ 选择器 '{}': {}", selector_str, src);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("❌ 读取响应失败: {}", e),
            }
        }
        Err(e) => println!("❌ 请求失败: {}", e),
    }

    println!("\n=== 测试结束 ===");
}
