use sqlx::SqlitePool;
use tauri::{AppHandle, Manager, State};

use super::{api, database};
use super::models::{LoginStatus, WereadAccount, WereadBook, WereadChapter, WereadDownload};

// --- 登录 ---

#[tauri::command]
pub async fn weread_get_account(
    pool: State<'_, SqlitePool>,
) -> Result<Option<WereadAccount>, String> {
    database::get_active_account(&pool).await
}

#[tauri::command]
pub async fn weread_check_cookie(
    pool: State<'_, SqlitePool>,
) -> Result<LoginStatus, String> {
    let account = match database::get_active_account(&pool).await? {
        Some(a) => a,
        None => return Ok(LoginStatus { logged_in: false, account: None }),
    };

    let client = api::build_authed_client(&account.cookies_json)?;
    let vid = &account.vid;

    match api::check_login(&client, vid).await? {
        Some(_) => Ok(LoginStatus {
            logged_in: true,
            account: Some(account),
        }),
        None => Ok(LoginStatus {
            logged_in: false,
            account: None,
        }),
    }
}

#[tauri::command]
pub async fn weread_save_login(
    cookies_json: String,
    pool: State<'_, SqlitePool>,
) -> Result<WereadAccount, String> {
    let client = api::build_authed_client(&cookies_json)?;

    let cookies: std::collections::HashMap<String, String> =
        serde_json::from_str(&cookies_json)
            .map_err(|e| format!("解析 cookies 失败: {}", e))?;

    let vid = cookies
        .get("wr_vid")
        .ok_or("cookies 中缺少 wr_vid")?;

    let info = api::check_login(&client, vid)
        .await?
        .ok_or("登录验证失败")?;

    database::upsert_account(
        &pool,
        &info.vid,
        &info.username,
        info.avatar_url.as_deref(),
        &cookies_json,
    )
    .await
}

#[tauri::command]
pub async fn weread_logout(pool: State<'_, SqlitePool>) -> Result<(), String> {
    let account = database::get_active_account(&pool)
        .await?
        .ok_or("没有已登录的账号")?;
    database::deactivate_account(&pool, &account.vid).await
}

#[tauri::command]
pub async fn weread_open_login(app: AppHandle) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;
    use tauri::WebviewUrl;

    if let Some(existing) = app.get_webview_window("weread-login") {
        existing.set_focus().ok();
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        &app,
        "weread-login",
        WebviewUrl::External("https://weread.qq.com/#login".parse().unwrap()),
    )
    .title("微信读书 - 扫码登录")
    .inner_size(500.0, 700.0)
    .build()
    .map_err(|e| format!("创建登录窗口失败: {}", e))?;

    let win_for_poll = window.clone();
    let app_handle = app.clone();
    std::thread::spawn(move || {
        println!("[weread] cookie polling thread started");
        std::thread::sleep(std::time::Duration::from_secs(5));
        for _ in 0..120 {
            std::thread::sleep(std::time::Duration::from_secs(3));

            if app_handle.get_webview_window("weread-login").is_none() {
                println!("[weread] login window closed, stopping poll");
                break;
            }

            let url: url::Url = "https://weread.qq.com/".parse().unwrap();
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                win_for_poll.cookies_for_url(url)
            })) {
                Ok(Ok(cookies)) => {
                    println!("[weread] got {} cookies", cookies.len());
                    let wr_vid = cookies.iter().find(|c| c.name() == "wr_vid");
                    if let Some(vid_cookie) = wr_vid {
                        println!("[weread] found wr_vid: {}", vid_cookie.value());
                        let mut cookies_map = std::collections::HashMap::new();
                        for c in &cookies {
                            cookies_map.insert(c.name().to_string(), c.value().to_string());
                        }
                        let cookies_json = serde_json::to_string(&cookies_map).unwrap_or_default();

                        let pool = app_handle.state::<SqlitePool>();
                        let pool_ref = pool.inner().clone();
                        tauri::async_runtime::block_on(async {
                            match save_login_from_cookies_str(&pool_ref, &cookies_json).await {
                                Ok(_) => println!("[weread] login saved successfully"),
                                Err(e) => eprintln!("[weread] save login failed: {}", e),
                            }
                        });

                        win_for_poll.close().ok();
                        break;
                    }
                }
                Ok(Err(e)) => {
                    println!("[weread] cookies_for_url error: {}", e);
                }
                Err(_) => {
                    println!("[weread] cookies_for_url panicked, window likely closed");
                    break;
                }
            }
        }
    });

    Ok(())
}

async fn save_login_from_cookies_str(pool: &SqlitePool, cookies_json: &str) -> Result<(), String> {
    let client = api::build_authed_client(cookies_json)?;

    let cookies: std::collections::HashMap<String, String> =
        serde_json::from_str(cookies_json)
            .map_err(|e| format!("解析 cookies 失败: {}", e))?;

    let vid = cookies
        .get("wr_vid")
        .ok_or("cookies 中缺少 wr_vid")?;

    let info = api::check_login(&client, vid)
        .await?
        .ok_or("登录验证失败")?;

    database::upsert_account(
        pool,
        &info.vid,
        &info.username,
        info.avatar_url.as_deref(),
        cookies_json,
    )
    .await?;

    Ok(())
}


// --- 书籍 ---

#[tauri::command]
pub async fn weread_sync_books(
    library_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<WereadBook>, String> {
    let account = get_active_account_or_err(&pool).await?;
    let client = api::build_authed_client(&account.cookies_json)?;

    let books = api::fetch_shelf(&client).await?;

    for book in &books {
        database::upsert_book(
            &pool,
            library_id,
            &book.book_id,
            &book.title,
            Some(&book.author),
            Some(&book.cover),
            Some(&book.intro),
            Some(&book.category),
            book.word_count,
            book.chapter_count,
        )
        .await?;
    }

    database::list_books(&pool, library_id).await
}

#[tauri::command]
pub async fn weread_list_books(
    library_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<WereadBook>, String> {
    database::list_books(&pool, library_id).await
}

#[tauri::command]
pub async fn weread_search_books(
    library_id: i64,
    query: String,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<WereadBook>, String> {
    database::search_books(&pool, library_id, &query).await
}

#[tauri::command]
pub async fn weread_get_book_detail(
    library_id: i64,
    book_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<WereadChapter>, String> {
    let account = get_active_account_or_err(&pool).await?;
    let client = api::build_authed_client(&account.cookies_json)?;

    let book = database::get_book_by_id(&pool, book_id)
        .await?
        .ok_or("书籍不存在")?;

    let detail = api::fetch_book_detail(&client, &book.book_id).await?;

    for (i, ch) in detail.chapters.iter().enumerate() {
        database::upsert_chapter(
            &pool,
            library_id,
            book_id,
            &ch.chapter_uid,
            &ch.title,
            ch.level,
            ch.word_count,
            i as i64,
        )
        .await?;
    }

    database::list_chapters(&pool, library_id, book_id).await
}

// --- 下载/导出 ---

#[tauri::command]
pub async fn weread_start_export(
    library_id: i64,
    book_id: i64,
    output_dir: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    database::upsert_download(&pool, library_id, book_id).await?;

    let pool_clone = pool.inner().clone();
    let output_path = std::path::PathBuf::from(&output_dir);

    tokio::spawn(async move {
        if let Err(e) = super::extractor::extract_book(&pool_clone, library_id, book_id).await {
            database::fail_download(&pool_clone, library_id, book_id, &e).await.ok();
            return;
        }

        match super::epub::build_epub(&pool_clone, library_id, book_id, &output_path).await {
            Ok(path) => {
                database::complete_download(
                    &pool_clone,
                    library_id,
                    book_id,
                    &path.to_string_lossy(),
                )
                .await
                .ok();
            }
            Err(e) => {
                database::fail_download(&pool_clone, library_id, book_id, &e).await.ok();
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn weread_get_export_progress(
    library_id: i64,
    book_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Option<WereadDownload>, String> {
    let downloads = database::list_downloads(&pool, library_id).await?;
    Ok(downloads.into_iter().find(|d| d.book_id == book_id))
}

#[tauri::command]
pub async fn weread_list_downloads(
    library_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<WereadDownload>, String> {
    database::list_downloads(&pool, library_id).await
}

// --- 辅助 ---

async fn get_active_account_or_err(pool: &SqlitePool) -> Result<WereadAccount, String> {
    database::get_active_account(pool)
        .await?
        .ok_or("请先登录微信读书".to_string())
}
