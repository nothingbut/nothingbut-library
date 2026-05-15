use headless_chrome::protocol::cdp::Network;
use headless_chrome::{Browser, LaunchOptions};
use sqlx::SqlitePool;
use std::time::Duration;

use super::{database, hash};

const HOOK_JS: &str = include_str!("hook.js");
const CHAPTER_DELAY_SECS: u64 = 5;
const PAGE_RENDER_WAIT_SECS: u64 = 3;

pub async fn extract_book(
    pool: &SqlitePool,
    library_id: i64,
    book_id: i64,
) -> Result<(), String> {
    let book = database::get_book_by_id(pool, book_id)
        .await?
        .ok_or("书籍不存在")?;

    let chapters = database::list_chapters(pool, library_id, book_id).await?;
    if chapters.is_empty() {
        return Err("没有章节数据，请先获取书籍详情".to_string());
    }

    let total = chapters.len() as i64;
    database::update_download_progress(pool, library_id, book_id, 0, total).await?;

    let account = database::get_active_account(pool)
        .await?
        .ok_or("请先登录微信读书")?;

    let cookies: std::collections::HashMap<String, String> =
        serde_json::from_str(&account.cookies_json)
            .map_err(|e| format!("解析 cookies 失败: {}", e))?;

    let pool_clone = pool.clone();
    let book_id_str = book.book_id.clone();
    let chapters_clone = chapters.clone();
    let cookies_clone = cookies.clone();

    tokio::task::spawn_blocking(move || {
        extract_chapters_sync(
            &pool_clone,
            library_id,
            book_id,
            &book_id_str,
            &chapters_clone,
            &cookies_clone,
            total,
        )
    })
    .await
    .map_err(|e| format!("提取任务失败: {}", e))?
}

fn extract_chapters_sync(
    pool: &SqlitePool,
    library_id: i64,
    book_id: i64,
    book_id_str: &str,
    chapters: &[super::models::WereadChapter],
    cookies: &std::collections::HashMap<String, String>,
    total: i64,
) -> Result<(), String> {
    println!("[weread-extract] starting Chrome...");

    let launch_options = LaunchOptions::default_builder()
        .headless(true)
        .sandbox(false)
        .window_size(Some((1200, 900)))
        .idle_browser_timeout(Duration::from_secs(300))
        .build()
        .map_err(|e| format!("配置 Chrome 失败: {}", e))?;

    let browser = Browser::new(launch_options)
        .map_err(|e| format!("启动 Chrome 失败: {}. 请确保安装了 Chrome/Chromium", e))?;

    println!("[weread-extract] Chrome started, creating tab...");

    let tab = browser
        .new_tab()
        .map_err(|e| format!("创建标签页失败: {}", e))?;

    println!("[weread-extract] navigating to weread.qq.com...");

    tab.navigate_to("https://weread.qq.com")
        .map_err(|e| format!("导航到首页失败: {}", e))?;

    std::thread::sleep(Duration::from_secs(3));

    println!("[weread-extract] setting cookies...");

    let mut cookie_params: Vec<Network::CookieParam> = cookies
        .iter()
        .map(|(key, value)| Network::CookieParam {
            name: key.clone(),
            value: value.clone(),
            url: Some("https://weread.qq.com".to_string()),
            domain: Some(".weread.qq.com".to_string()),
            path: Some("/".to_string()),
            secure: None,
            http_only: None,
            same_site: None,
            expires: None,
            priority: None,
            same_party: None,
            source_scheme: None,
            source_port: None,
            partition_key: None,
        })
        .collect();

    cookie_params.push(Network::CookieParam {
        name: "wr_useHorizonReader".to_string(),
        value: "0".to_string(),
        url: Some("https://weread.qq.com".to_string()),
        domain: Some(".weread.qq.com".to_string()),
        path: Some("/".to_string()),
        secure: None,
        http_only: None,
        same_site: None,
        expires: None,
        priority: None,
        same_party: None,
        source_scheme: None,
        source_port: None,
        partition_key: None,
    });

    tab.set_cookies(cookie_params)
        .map_err(|e| format!("设置 cookies 失败: {}", e))?;

    println!("[weread-extract] cookies set, starting chapter extraction...");

    for (i, chapter) in chapters.iter().enumerate() {
        if chapter.content_md.is_some() {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(database::update_download_progress(
                pool, library_id, book_id, (i + 1) as i64, total,
            ))
            .ok();
            continue;
        }

        let chapter_hash = hash::wr_hash(&chapter.chapter_uid);
        let url = format!(
            "https://weread.qq.com/web/reader/{}k{}",
            book_id_str, chapter_hash
        );

        println!("[weread] extracting chapter {}/{}: {}", i + 1, total, chapter.title);

        tab.navigate_to(&url)
            .map_err(|e| format!("导航到章节失败: {}", e))?;

        std::thread::sleep(Duration::from_secs(PAGE_RENDER_WAIT_SECS));

        tab.evaluate(HOOK_JS, false)
            .map_err(|e| format!("注入 Hook 失败: {}", e))?;

        std::thread::sleep(Duration::from_secs(2));

        tab.evaluate("window.__wereadMarkComplete();", false).ok();

        let multi_page_js = r#"
            (function() {
                var maxPages = 50;
                var pages = 0;
                while (window.__wereadHasNextPage && window.__wereadHasNextPage() && pages < maxPages) {
                    window.__wereadClickNextPage();
                    pages++;
                }
                window.__wereadMarkComplete();
                return pages;
            })()
        "#;
        tab.evaluate(multi_page_js, false).ok();

        std::thread::sleep(Duration::from_secs(1));

        let content_result = tab.evaluate(
            "JSON.stringify(window.__wereadExtracted ? window.__wereadExtracted.content : '')",
            false,
        );

        let content = match content_result {
            Ok(result) => result
                .value
                .and_then(|v| {
                    v.as_str()
                        .map(|s| s.to_string())
                        .or_else(|| serde_json::from_value::<String>(v).ok())
                })
                .unwrap_or_default(),
            Err(e) => {
                eprintln!("[weread] 提取章节内容失败: {}", e);
                String::new()
            }
        };

        let final_content = if content.is_empty() {
            format!("<!-- 提取失败: {} -->\n", chapter.title)
        } else {
            content
        };

        let rt = tokio::runtime::Handle::current();
        rt.block_on(database::update_chapter_content(
            pool,
            library_id,
            book_id,
            &chapter.chapter_uid,
            &final_content,
        ))
        .map_err(|e| format!("保存章节内容失败: {}", e))?;

        rt.block_on(database::update_download_progress(
            pool, library_id, book_id, (i + 1) as i64, total,
        ))
        .ok();

        if i < chapters.len() - 1 {
            std::thread::sleep(Duration::from_secs(CHAPTER_DELAY_SECS));
        }
    }

    Ok(())
}
