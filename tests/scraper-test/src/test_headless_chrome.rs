/// 测试方案3：Headless Chrome访问书籍详情页
/// 使用已知的book ID: 1010400217 (黎明之剑)

use headless_chrome::{Browser, LaunchOptions};
use scraper::{Html, Selector};
use std::time::Duration;

fn main() {
    println!("=== 测试方案3：Headless Chrome访问书籍详情页 ===\n");

    let book_id = "1010400217"; // 黎明之剑的book ID
    let book_url = format!("https://www.qidian.com/book/{}/", book_id);

    println!("测试数据：");
    println!("  Book ID: {}", book_id);
    println!("  URL: {}\n", book_url);

    println!("🌐 启动Headless Chrome...");

    let browser = match Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .build()
            .expect("Failed to build launch options")
    ) {
        Ok(b) => {
            println!("✅ Chrome启动成功\n");
            b
        }
        Err(e) => {
            println!("❌ Chrome启动失败: {}\n", e);
            println!("提示：首次运行会下载Chromium，请确保网络正常");
            return;
        }
    };

    match browser.new_tab() {
        Ok(tab) => {
            println!("📡 访问URL: {}", book_url);

            if let Err(e) = tab.navigate_to(&book_url) {
                println!("❌ 导航失败: {}", e);
                return;
            }

            println!("⏳ 等待页面加载...");
            if let Err(e) = tab.wait_until_navigated() {
                println!("❌ 等待导航失败: {}", e);
                return;
            }

            // 等待JavaScript执行
            println!("⏳ 等待JavaScript执行（3秒）...");
            std::thread::sleep(Duration::from_secs(3));

            match tab.get_content() {
                Ok(html) => {
                    println!("✅ 获取页面内容成功");
                    println!("HTML长度: {} bytes", html.len());
                    println!("HTML前500字符:");
                    println!("{}\n", &html.chars().take(500).collect::<String>());

                    // 检查是否是验证码页面
                    if html.contains("TencentCaptcha") || html.contains("captcha") {
                        println!("⚠️  检测到验证码页面！");
                        println!("起点的WAF检测到了headless Chrome\n");
                        return;
                    }

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
                                        if let Some(src) = element.value().attr("src") {
                                            println!("  ✅ 选择器 '{}': {}", selector_str, src);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => println!("❌ 获取内容失败: {}", e),
            }
        }
        Err(e) => println!("❌ 创建标签页失败: {}", e),
    }

    println!("\n=== 测试结束 ===");
}
