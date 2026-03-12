use std::path::Path;
use reqwest;
use scraper::{Html, Selector};
use regex::Regex;
use serde_json::Value;
use headless_chrome::{Browser, LaunchOptions};
use std::time::Duration;
use crate::errors::{AppError, AppResult};

/// Metadata fetched from website
#[derive(Debug, Clone)]
pub struct ScrapedMetadata {
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub author: Option<String>,
    pub category: Option<String>,
}

/// Download and process cover image
pub async fn download_cover(
    cover_url: &str,
    book_dir: &Path,
) -> AppResult<String> {
    // Download image
    let response = reqwest::get(cover_url)
        .await
        .map_err(|e| AppError::Io(format!("Failed to download cover: {}", e)))?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::Io(format!("Failed to read cover bytes: {}", e)))?;

    // Load image
    let img = image::load_from_memory(&bytes)
        .map_err(|e| AppError::Io(format!("Failed to decode image: {}", e)))?;

    // Resize if too large (max width 800px, maintain aspect ratio)
    let img = if img.width() > 800 {
        img.resize(800, img.height(), image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // Save as JPEG
    let cover_path = book_dir.join("cover.jpg");
    img.save_with_format(&cover_path, image::ImageFormat::Jpeg)
        .map_err(|e| AppError::Io(format!("Failed to save cover: {}", e)))?;

    // Return relative path from workspace
    let relative_path = book_dir
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{}/cover.jpg", name))
        .ok_or_else(|| AppError::Io("Invalid book directory path".to_string()))?;

    Ok(relative_path)
}

/// Try to extract book ID from a Qidian URL
fn extract_qidian_book_id(text: &str) -> Option<String> {
    // Match patterns like: https://www.qidian.com/book/1010400217
    let re = Regex::new(r"qidian\.com/book/(\d+)").ok()?;
    re.captures(text)?.get(1).map(|m| m.as_str().to_string())
}

/// Fetch book details directly from Qidian book page
async fn fetch_qidian_book_details(client: &reqwest::Client, book_id: &str) -> AppResult<ScrapedMetadata> {
    let book_url = format!("https://www.qidian.com/book/{}/", book_id);
    println!("Fetching book details from: {}", book_url);

    let response = client
        .get(&book_url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .header("Referer", "https://www.qidian.com/")
        .send()
        .await
        .map_err(|e| AppError::Io(format!("Failed to fetch book page: {}", e)))?;

    let html = response
        .text()
        .await
        .map_err(|e| AppError::Io(format!("Failed to read response: {}", e)))?;

    println!("Book page HTML length: {}", html.len());

    if html.len() < 1000 {
        println!("Warning: HTML response too short, might be blocked or redirected");
        return Err(AppError::Io(
            "起点返回的页面内容异常（可能需要登录或被反爬虫阻止）".to_string()
        ));
    }

    let document = Html::parse_document(&html);

    // Try multiple selector patterns for different page structures
    let selector_sets = vec![
        // Pattern 1: Standard book page
        (
            ".book-info .writer, .book-info .author, h1 em.author",
            "#bookImg img, .book-img img",
            ".book-intro p, .intro, #intro p",
            ".book-info .tag a, .book-info em.blue, span.blue",
        ),
        // Pattern 2: Mobile/alternative layout
        (
            ".author-name, .book-author",
            ".cover img, .book-cover img",
            ".intro-text, .book-desc",
            ".book-tag, .category",
        ),
    ];

    let mut description = None;
    let mut cover_url = None;
    let mut author = None;
    let mut category = None;

    for (author_sel, cover_sel, intro_sel, cat_sel) in selector_sets {
        // Try author
        if author.is_none() {
            if let Ok(selector) = Selector::parse(author_sel) {
                author = document
                    .select(&selector)
                    .next()
                    .map(|elem| elem.text().collect::<String>().trim().to_string())
                    .filter(|s| !s.is_empty() && s != "作者");

                if let Some(ref a) = author {
                    println!("Found author: {}", a);
                }
            }
        }

        // Try cover
        if cover_url.is_none() {
            if let Ok(selector) = Selector::parse(cover_sel) {
                cover_url = document
                    .select(&selector)
                    .next()
                    .and_then(|img| img.value().attr("src").or_else(|| img.value().attr("data-src")))
                    .map(|src| {
                        if src.starts_with("//") {
                            format!("https:{}", src)
                        } else if src.starts_with("/") {
                            format!("https://www.qidian.com{}", src)
                        } else {
                            src.to_string()
                        }
                    });

                if let Some(ref url) = cover_url {
                    println!("Found cover: {}", url);
                }
            }
        }

        // Try description
        if description.is_none() {
            if let Ok(selector) = Selector::parse(intro_sel) {
                let desc = document
                    .select(&selector)
                    .map(|elem| elem.text().collect::<String>().trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");

                if !desc.is_empty() {
                    description = Some(desc);
                    println!("Found description length: {}", description.as_ref().unwrap().len());
                }
            }
        }

        // Try category
        if category.is_none() {
            if let Ok(selector) = Selector::parse(cat_sel) {
                category = document
                    .select(&selector)
                    .filter_map(|elem| {
                        let text = elem.text().collect::<String>().trim().to_string();
                        if !text.is_empty() && text != "作者" && !text.contains("点击") {
                            Some(text)
                        } else {
                            None
                        }
                    })
                    .next();

                if let Some(ref c) = category {
                    println!("Found category: {}", c);
                }
            }
        }
    }

    // Check if we got any useful data
    if description.is_none() && author.is_none() && cover_url.is_none() {
        return Err(AppError::NotFound(
            "无法从起点书籍页面提取信息。页面结构可能已更改。".to_string()
        ));
    }

    Ok(ScrapedMetadata {
        description,
        cover_url,
        author,
        category,
    })
}

/// Fetch URL content using headless Chrome to bypass WAF
fn fetch_with_headless_chrome(url: &str) -> AppResult<String> {
    println!("🌐 启动headless Chrome...");

    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(true)
            .build()
            .map_err(|e| AppError::Io(format!("Failed to build launch options: {}", e)))?
    ).map_err(|e| AppError::Io(format!("Failed to launch Chrome: {}", e)))?;

    println!("✅ Chrome启动成功");
    println!("📡 访问URL: {}", url);

    let tab = browser.new_tab().map_err(|e| {
        AppError::Io(format!("Failed to create new tab: {}", e))
    })?;

    tab.navigate_to(url).map_err(|e| {
        AppError::Io(format!("Failed to navigate: {}", e))
    })?;

    println!("⏳ 等待页面加载...");
    tab.wait_until_navigated().map_err(|e| {
        AppError::Io(format!("Failed to wait for navigation: {}", e))
    })?;

    // Wait for JavaScript to execute
    std::thread::sleep(Duration::from_secs(3));
    println!("✅ 页面加载完成");

    let content = tab.get_content().map_err(|e| {
        AppError::Io(format!("Failed to get page content: {}", e))
    })?;

    println!("📄 获取页面内容，长度: {} bytes", content.len());

    Ok(content)
}

/// Scrape metadata from Qidian using JSON API (more reliable than HTML scraping)
async fn scrape_qidian_api(_client: &reqwest::Client, title: &str, author: Option<&str>) -> AppResult<ScrapedMetadata> {
    println!("=== 起点API调用开始 ===");
    println!("输入书名: {}", title);
    println!("输入作者: {:?}", author);

    let search_query = if let Some(auth) = author {
        format!("{} {}", title, auth)
    } else {
        title.to_string()
    };

    let api_url = format!(
        "https://www.qidian.com/ajax/search?kw={}",
        urlencoding::encode(&search_query)
    );

    println!("搜索关键词: {}", search_query);
    println!("API URL: {}", api_url);
    println!("使用headless Chrome绕过WAF...");

    // Use headless Chrome to bypass WAF
    let response_text = tokio::task::spawn_blocking(move || {
        fetch_with_headless_chrome(&api_url)
    })
    .await
    .map_err(|e| AppError::Io(format!("Task join error: {}", e)))?
    .map_err(|e| {
        println!("❌ Headless Chrome失败: {}", e);
        e
    })?;

    println!("✅ 获取响应成功");
    println!("响应文本长度: {} bytes", response_text.len());
    println!("响应内容前500字符: {}", &response_text.chars().take(500).collect::<String>());

    let json: Value = serde_json::from_str(&response_text)
        .map_err(|e| {
            println!("❌ JSON解析失败: {}", e);
            println!("完整响应文本: {}", response_text);
            AppError::Io(format!("Failed to parse JSON response: {}", e))
        })?;

    println!("✅ JSON解析成功");
    println!("JSON结构: {}", serde_json::to_string_pretty(&json).unwrap_or_default());

    // Parse JSON response to extract book list
    println!("开始解析书籍列表...");
    let books = json
        .get("data")
        .and_then(|data| data.as_array())
        .ok_or_else(|| {
            println!("❌ JSON中没有找到data数组");
            println!("JSON keys: {:?}", json.as_object().map(|o| o.keys().collect::<Vec<_>>()));
            AppError::NotFound("API返回数据格式异常".to_string())
        })?;

    println!("找到 {} 本书", books.len());

    if books.is_empty() {
        println!("❌ 搜索结果为空");
        return Err(AppError::NotFound(format!(
            "在起点未找到《{}》的搜索结果",
            title
        )));
    }

    // Find matching book
    println!("开始匹配书籍...");
    for (index, book) in books.iter().enumerate() {
        println!("--- 检查第 {} 本书 ---", index + 1);

        let book_title = book.get("bookName")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let book_author = book.get("authorName")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        println!("书名: {}", book_title);
        println!("作者: {}", book_author);

        // Match title
        let title_match = book_title.contains(title) || title.contains(book_title);
        println!("书名匹配: {} (输入: {}, API返回: {})", title_match, title, book_title);

        if !title_match {
            println!("❌ 书名不匹配，跳过");
            continue;
        }

        // Match author if provided
        if let Some(auth) = author {
            let author_match = book_author.contains(auth) || auth.contains(book_author);
            println!("作者匹配: {} (输入: {}, API返回: {})", author_match, auth, book_author);

            if !author_match {
                println!("❌ 作者不匹配，跳过");
                continue;
            }
        } else {
            println!("未提供作者，跳过作者匹配");
        }

        println!("✅ 匹配成功！开始提取元数据...");

        // Extract metadata from JSON
        let description = book.get("desc")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        println!("简介: {:?}", description.as_ref().map(|s| {
            if s.len() > 100 {
                format!("{}...", &s[..100])
            } else {
                s.clone()
            }
        }));

        let cover_url = book.get("img")
            .and_then(|v| v.as_str())
            .map(|src| {
                let full_url = if src.starts_with("//") {
                    format!("https:{}", src)
                } else if src.starts_with("/") {
                    format!("https://www.qidian.com{}", src)
                } else {
                    src.to_string()
                };
                println!("封面URL: {}", full_url);
                full_url
            });

        let author_text = book.get("authorName")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        println!("作者: {:?}", author_text);

        let category = book.get("cateName")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        println!("分类: {:?}", category);

        println!("=== 起点API调用成功 ===\n");

        return Ok(ScrapedMetadata {
            description,
            cover_url,
            author: author_text,
            category,
        });
    }

    println!("❌ 遍历完所有书籍，未找到匹配项");
    println!("=== 起点API调用结束 ===\n");

    Err(AppError::NotFound(format!(
        "在起点API结果中未找到匹配的《{}》",
        title
    )))
}

/// Scrape metadata from Qidian (起点)
pub async fn scrape_qidian(title: &str, author: Option<&str>) -> AppResult<ScrapedMetadata> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .cookie_store(true)  // Enable automatic cookie handling
        .build()
        .map_err(|e| AppError::Io(format!("Failed to create HTTP client: {}", e)))?;

    // Check if title contains a Qidian URL or book ID
    if let Some(book_id) = extract_qidian_book_id(title) {
        println!("Detected Qidian book ID: {}", book_id);
        return fetch_qidian_book_details(&client, &book_id).await;
    }

    // Try JSON API first (more reliable than HTML scraping)
    println!("Trying Qidian JSON API first...");
    match scrape_qidian_api(&client, title, author).await {
        Ok(metadata) => {
            println!("Successfully fetched from JSON API");
            return Ok(metadata);
        }
        Err(e) => {
            println!("JSON API failed: {}, falling back to HTML scraping", e);
        }
    }

    // Fallback to HTML scraping
    println!("Falling back to HTML scraping...");
    let search_query = if let Some(auth) = author {
        format!("{} {}", title, auth)
    } else {
        title.to_string()
    };

    // Use Qidian's search page (HTML response)
    let api_url = format!(
        "https://www.qidian.com/so/{}",
        urlencoding::encode(&search_query)
    );

    println!("Searching Qidian HTML: {}", api_url);

    let response = client
        .get(&api_url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml")
        .header("Accept-Language", "zh-CN,zh;q=0.9,en;q=0.8")
        .send()
        .await
        .map_err(|e| AppError::Io(format!("Failed to search Qidian: {}", e)))?;

    let html = response
        .text()
        .await
        .map_err(|e| AppError::Io(format!("Failed to read response: {}", e)))?;

    println!("Response length: {}", html.len());

    let document = Html::parse_document(&html);

    // Try multiple selector patterns for Qidian's changing HTML structure
    let selectors = vec![
        // Pattern 1: res-book-item
        (".res-book-item", ".book-mid-info h4 a", ".book-mid-info .author .name", ".book-img-box img", ".book-mid-info .intro", ".book-mid-info p.author a"),
        // Pattern 2: search result item
        (".search-result-item", ".book-info h4 a", ".book-info .author", ".book-cover img", ".book-info .intro", ".book-info .author a"),
        // Pattern 3: result-item
        (".result-item", ".info h3 a", ".info .author", ".img img", ".info .intro", ".info .type"),
    ];

    for (item_sel, title_sel, author_sel, cover_sel, intro_sel, cat_sel) in selectors {
        let book_selector = match Selector::parse(item_sel) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let title_selector = match Selector::parse(title_sel) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let author_selector = match Selector::parse(author_sel) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let cover_selector = match Selector::parse(cover_sel) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let intro_selector = match Selector::parse(intro_sel) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let category_selector = match Selector::parse(cat_sel) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for book_item in document.select(&book_selector) {
            // Check title match
            if let Some(title_elem) = book_item.select(&title_selector).next() {
                let book_title = title_elem.text().collect::<String>().trim().to_string();

                println!("Found book: {}", book_title);

                // Simple title matching (contains)
                if !book_title.contains(title) && !title.contains(&book_title) {
                    continue;
                }

                // Check author if provided
                if let Some(auth) = author {
                    if let Some(author_elem) = book_item.select(&author_selector).next() {
                        let book_author = author_elem.text().collect::<String>().trim().to_string();
                        if !book_author.contains(auth) && !auth.contains(&book_author) {
                            continue;
                        }
                    }
                }

                // Extract metadata
                let cover_url = book_item
                    .select(&cover_selector)
                    .next()
                    .and_then(|img| img.value().attr("src").or_else(|| img.value().attr("data-src")))
                    .map(|src| {
                        if src.starts_with("//") {
                            format!("https:{}", src)
                        } else if src.starts_with("/") {
                            format!("https://www.qidian.com{}", src)
                        } else {
                            src.to_string()
                        }
                    });

                let description = book_item
                    .select(&intro_selector)
                    .next()
                    .map(|elem| elem.text().collect::<String>().trim().to_string());

                let author_text = book_item
                    .select(&author_selector)
                    .next()
                    .map(|elem| elem.text().collect::<String>().trim().to_string());

                // Extract category
                let category = book_item
                    .select(&category_selector)
                    .filter_map(|elem| {
                        let text = elem.text().collect::<String>().trim().to_string();
                        // Filter out author names and other non-category text
                        let author_str = author_text.as_ref().map(|s| s.as_str()).unwrap_or("");
                        if !text.is_empty() && text != author_str && text != "作者" {
                            Some(text)
                        } else {
                            None
                        }
                    })
                    .next();

                return Ok(ScrapedMetadata {
                    description,
                    cover_url,
                    author: author_text,
                    category,
                });
            }
        }
    }

    Err(AppError::NotFound(format!(
        "无法在起点找到《{}》。\n\n建议：\n1. 在书名栏中直接输入起点书籍URL（如：https://www.qidian.com/book/1010400217）\n2. 或确保书名和作者完全正确",
        title
    )))
}

/// Scrape metadata from Duyuedu (独阅读)
pub async fn scrape_duyuedu(title: &str, author: Option<&str>) -> AppResult<ScrapedMetadata> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .build()
        .map_err(|e| AppError::Io(format!("Failed to create HTTP client: {}", e)))?;

    // Search for the book
    let search_query = if let Some(auth) = author {
        format!("{} {}", title, auth)
    } else {
        title.to_string()
    };

    let search_url = format!(
        "https://www.duyuedu.com/search.html?q={}",
        urlencoding::encode(&search_query)
    );

    let response = client
        .get(&search_url)
        .send()
        .await
        .map_err(|e| AppError::Io(format!("Failed to search Duyuedu: {}", e)))?;

    let html = response
        .text()
        .await
        .map_err(|e| AppError::Io(format!("Failed to read response: {}", e)))?;

    let document = Html::parse_document(&html);

    // Try to find the book in search results
    let book_selector = Selector::parse(".book-item, .result-item").unwrap();
    let title_selector = Selector::parse(".book-title a, .title a, h3 a").unwrap();
    let author_selector = Selector::parse(".book-author, .author").unwrap();
    let cover_selector = Selector::parse(".book-cover img, .cover img").unwrap();
    let intro_selector = Selector::parse(".book-intro, .intro, .description").unwrap();
    let category_selector = Selector::parse(".book-category, .category, .book-type").unwrap();

    for book_item in document.select(&book_selector) {
        // Check title match
        if let Some(title_elem) = book_item.select(&title_selector).next() {
            let book_title = title_elem.text().collect::<String>().trim().to_string();

            // Simple title matching
            if !book_title.contains(title) && !title.contains(&book_title) {
                continue;
            }

            // Check author if provided
            if let Some(auth) = author {
                if let Some(author_elem) = book_item.select(&author_selector).next() {
                    let book_author = author_elem.text().collect::<String>().trim().to_string();
                    if !book_author.contains(auth) && !auth.contains(&book_author) {
                        continue;
                    }
                }
            }

            // Extract metadata
            let cover_url = book_item
                .select(&cover_selector)
                .next()
                .and_then(|img| img.value().attr("src"))
                .map(|src| {
                    if src.starts_with("//") {
                        format!("https:{}", src)
                    } else if src.starts_with("/") {
                        format!("https://www.duyuedu.com{}", src)
                    } else {
                        src.to_string()
                    }
                });

            let description = book_item
                .select(&intro_selector)
                .next()
                .map(|elem| elem.text().collect::<String>().trim().to_string());

            let author = book_item
                .select(&author_selector)
                .next()
                .map(|elem| elem.text().collect::<String>().trim().to_string());

            // Extract category
            let category = book_item
                .select(&category_selector)
                .next()
                .map(|elem| elem.text().collect::<String>().trim().to_string());

            return Ok(ScrapedMetadata {
                description,
                cover_url,
                author,
                category,
            });
        }
    }

    Err(AppError::NotFound("Book not found on Duyuedu".to_string()))
}

/// Scrape metadata based on source site
pub async fn scrape_metadata(
    source_site: &str,
    title: &str,
    author: Option<&str>,
) -> AppResult<ScrapedMetadata> {
    match source_site {
        "起点" => scrape_qidian(title, author).await,
        "独阅读" => scrape_duyuedu(title, author).await,
        _ => Err(AppError::Validation(format!(
            "Scraping not supported for source site: {}",
            source_site
        ))),
    }
}
