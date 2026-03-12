/// 测试方案4：使用Chromiumoxide的Stealth模式
/// Chromiumoxide是比headless_chrome更现代的Rust Chrome自动化库
/// 内置stealth模式可以隐藏headless浏览器特征
///
/// 测试数据：黎明之剑 / 远瞳 / Book ID: 1010400217

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use futures::StreamExt;
use scraper::{Html, Selector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试方案4：Chromiumoxide Stealth模式 ===\n");

    let book_id = "1010400217"; // 黎明之剑
    let book_url = format!("https://www.qidian.com/book/{}/", book_id);

    println!("测试数据：");
    println!("  Book ID: {}", book_id);
    println!("  URL: {}\n", book_url);

    println!("🌐 启动Chromiumoxide浏览器（Stealth模式）...");

    // 配置浏览器
    let (mut browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .no_sandbox()
            .window_size(1920, 1080)
            .build()?
    ).await?;

    // 启动事件处理器
    let handle = tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(e) = event {
                eprintln!("Browser event error: {}", e);
            }
        }
    });

    println!("✅ 浏览器启动成功\n");

    // 创建新页面
    let page = browser.new_page("about:blank").await?;

    // ⭐ 关键：启用Stealth模式
    println!("🔒 启用Stealth模式（隐藏自动化特征）...");
    page.enable_stealth_mode_with_agent(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    ).await?;
    println!("✅ Stealth模式已启用\n");

    // 验证webdriver属性是否被隐藏
    let webdriver_hidden: bool = page
        .evaluate("navigator.webdriver === undefined")
        .await?
        .into_value()?;
    println!("navigator.webdriver是否隐藏: {}\n", webdriver_hidden);

    // 访问起点书籍页
    println!("📡 访问URL: {}", book_url);
    page.goto(&book_url).await?;

    println!("⏳ 等待页面加载...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // 获取页面内容
    let html = page.content().await?;
    println!("✅ 获取页面内容成功");
    println!("HTML长度: {} bytes\n", html.len());

    println!("HTML前500字符:");
    println!("{}\n", &html.chars().take(500).collect::<String>());

    // 检查是否是验证码页面
    if html.contains("TencentCaptcha") || html.contains("captcha") {
        println!("❌ 仍然检测到验证码页面！");
        println!("Stealth模式未能完全绕过WAF\n");
    } else if html.len() < 1000 {
        println!("⚠️  HTML太短，可能被WAF拦截\n");
    } else {
        println!("✅ 页面加载正常，开始解析...\n");

        // 保存截图用于调试
        page.save_screenshot(
            chromiumoxide::page::ScreenshotParams::builder()
                .format(CaptureScreenshotFormat::Png)
                .full_page(false)
                .build(),
            "chromiumoxide-stealth.png"
        ).await?;
        println!("📸 已保存截图: chromiumoxide-stealth.png\n");

        // 解析HTML
        let document = Html::parse_document(&html);

        let selectors = vec![
            ("书名", vec!["h1", ".book-info h1", "meta[property='og:title']"]),
            ("作者", vec![".writer", ".author-name-text", ".book-info .writer"]),
            ("封面", vec!["#bookImg img", ".book-img img"]),
            ("简介", vec![".book-intro p", "#intro p", ".intro"]),
            ("分类", vec![".tag", "a.red", ".book-info .tag"]),
        ];

        for (name, selector_list) in selectors {
            println!("尝试提取【{}】:", name);
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
            println!();
        }
    }

    // 清理
    browser.close().await?;
    handle.await?;

    println!("=== 测试结束 ===");
    Ok(())
}
