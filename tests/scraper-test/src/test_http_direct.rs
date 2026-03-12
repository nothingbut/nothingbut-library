/// 测试方案1：直接HTTP请求搜索API
/// 测试数据：黎明之剑、远瞳

use reqwest;
use serde_json::Value;

fn main() {
    println!("=== 测试方案1：直接HTTP请求搜索API ===\n");

    let title = "黎明之剑";
    let author = "远瞳";
    let search_query = format!("{} {}", title, author);

    let api_url = format!(
        "https://www.qidian.com/ajax/search?kw={}",
        urlencoding::encode(&search_query)
    );

    println!("测试数据：");
    println!("  书名: {}", title);
    println!("  作者: {}", author);
    println!("  API URL: {}\n", api_url);

    println!("发送请求...");

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .cookie_store(true)
        .build()
        .expect("Failed to build client");

    match client.get(&api_url)
        .header("Accept", "application/json, text/plain, */*")
        .header("X-Requested-With", "XMLHttpRequest")
        .header("Referer", "https://www.qidian.com/")
        .send()
    {
        Ok(response) => {
            println!("✅ 收到响应");
            println!("状态码: {}", response.status());
            println!("响应头: {:#?}\n", response.headers());

            match response.text() {
                Ok(text) => {
                    println!("响应文本长度: {} bytes", text.len());
                    println!("响应内容前500字符:");
                    println!("{}\n", &text.chars().take(500).collect::<String>());

                    // 尝试解析JSON
                    match serde_json::from_str::<Value>(&text) {
                        Ok(json) => {
                            println!("✅ JSON解析成功！");
                            println!("{}\n", serde_json::to_string_pretty(&json).unwrap());

                            // 提取书籍信息
                            if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
                                println!("找到 {} 本书：", data.len());
                                for (i, book) in data.iter().enumerate() {
                                    println!("\n书籍 {}:", i + 1);
                                    println!("  书名: {}", book.get("bookName").and_then(|v| v.as_str()).unwrap_or("N/A"));
                                    println!("  作者: {}", book.get("authorName").and_then(|v| v.as_str()).unwrap_or("N/A"));
                                    println!("  分类: {}", book.get("cateName").and_then(|v| v.as_str()).unwrap_or("N/A"));
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ JSON解析失败: {}", e);
                            println!("这不是JSON数据，可能是HTML或验证页面");
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
