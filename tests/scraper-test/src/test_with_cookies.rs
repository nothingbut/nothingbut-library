/// 测试方案5：使用浏览器Cookies
///
/// 灵感来源：Novel-Downloader项目
/// - 用户在浏览器中访问起点并完成验证
/// - 用户复制cookies到配置文件
/// - 程序使用这些cookies访问API
///
/// 测试步骤：
/// 1. 在浏览器中访问 https://www.qidian.com
/// 2. 完成任何验证
/// 3. 打开开发者工具 > Application > Cookies
/// 4. 复制相关cookies
/// 5. 运行此测试

use reqwest;
use serde_json::Value;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 测试方案5：使用浏览器Cookies ===\n");

    let book_id = "1010400217"; // 黎明之剑

    println!("测试说明：");
    println!("此测试需要您提供浏览器中的cookies。\n");
    println!("获取cookies步骤：");
    println!("1. 在浏览器中访问 https://www.qidian.com");
    println!("2. 完成任何验证（如果有）");
    println!("3. 按F12打开开发者工具");
    println!("4. 进入 Application > Cookies > https://www.qidian.com");
    println!("5. 复制以下cookies的值：");
    println!("   - _csrfToken");
    println!("   - newstatisticUUID");
    println!("   - 其他相关cookies\n");

    // TODO: 从配置文件或环境变量读取cookies
    // 这里演示如何使用cookies
    let cookies = load_cookies_from_config();

    if cookies.is_empty() {
        println!("⚠️  未提供cookies，将尝试不带cookies访问（预计失败）\n");
    } else {
        println!("✅ 加载了 {} 个cookies\n", cookies.len());
    }

    // 测试1：访问书籍详情页
    println!("测试1: 访问书籍详情页");
    test_book_page(&cookies, book_id).await?;

    println!("\n");

    // 测试2：访问搜索API
    println!("测试2: 访问搜索API");
    test_search_api(&cookies, "黎明之剑", "远瞳").await?;

    println!("\n=== 测试结束 ===");
    println!("\n💡 提示：");
    println!("如果测试失败，请确保：");
    println!("1. Cookies是最新的（未过期）");
    println!("2. 在浏览器中能正常访问起点");
    println!("3. 复制了所有必要的cookies");

    Ok(())
}

async fn test_book_page(
    cookies: &HashMap<String, String>,
    book_id: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let book_url = format!("https://www.qidian.com/book/{}/", book_id);
    println!("  URL: {}", book_url);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let mut request = client.get(&book_url);

    // 添加cookies
    let cookie_header = build_cookie_header(cookies);
    if !cookie_header.is_empty() {
        request = request.header("Cookie", cookie_header);
        println!("  添加了cookies");
    }

    let response = request
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .send()
        .await?;

    println!("  状态码: {}", response.status());

    let html = response.text().await?;
    println!("  HTML长度: {} bytes", html.len());

    if html.contains("TencentCaptcha") || html.contains("captcha") {
        println!("  ❌ 仍然遇到验证码页面");
    } else if html.len() < 1000 {
        println!("  ❌ 页面太短，可能被拦截");
    } else if html.contains("book-info") || html.contains("黎明之剑") {
        println!("  ✅ 成功访问书籍页面！");
        println!("  找到书籍信息标记");
    } else {
        println!("  ⚠️  页面内容异常");
        println!("  前200字符: {}", &html.chars().take(200).collect::<String>());
    }

    Ok(())
}

async fn test_search_api(
    cookies: &HashMap<String, String>,
    title: &str,
    author: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let search_query = format!("{} {}", title, author);
    let api_url = format!(
        "https://www.qidian.com/ajax/search?kw={}",
        urlencoding::encode(&search_query)
    );

    println!("  URL: {}", api_url);

    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .build()?;

    let mut request = client.get(&api_url);

    // 添加cookies
    let cookie_header = build_cookie_header(cookies);
    if !cookie_header.is_empty() {
        request = request.header("Cookie", cookie_header);
        println!("  添加了cookies");
    }

    let response = request
        .header("Accept", "application/json, text/plain, */*")
        .header("X-Requested-With", "XMLHttpRequest")
        .header("Referer", "https://www.qidian.com/")
        .send()
        .await?;

    println!("  状态码: {}", response.status());

    let text = response.text().await?;
    println!("  响应长度: {} bytes", text.len());

    // 尝试解析JSON
    match serde_json::from_str::<Value>(&text) {
        Ok(json) => {
            println!("  ✅ 成功获取JSON数据！");
            if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                println!("  找到 {} 本书", data.len());
                for (i, book) in data.iter().take(3).enumerate() {
                    println!("  书籍{}: {}", i + 1,
                        book.get("bookName").and_then(|v| v.as_str()).unwrap_or("N/A")
                    );
                }
            }
        }
        Err(_) => {
            println!("  ❌ 响应不是JSON");
            println!("  前200字符: {}", &text.chars().take(200).collect::<String>());
        }
    }

    Ok(())
}

fn load_cookies_from_config() -> HashMap<String, String> {
    let mut cookies = HashMap::new();

    // 尝试从环境变量读取
    if let Ok(cookie_str) = std::env::var("QIDIAN_COOKIES") {
        println!("从环境变量QIDIAN_COOKIES加载cookies");
        for pair in cookie_str.split(';') {
            let parts: Vec<&str> = pair.trim().splitn(2, '=').collect();
            if parts.len() == 2 {
                cookies.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }
    }

    // 尝试从配置文件读取
    if let Ok(content) = std::fs::read_to_string("qidian_cookies.txt") {
        println!("从文件qidian_cookies.txt加载cookies");
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() == 2 {
                cookies.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
            }
        }
    }

    cookies
}

fn build_cookie_header(cookies: &HashMap<String, String>) -> String {
    cookies.iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ")
}
