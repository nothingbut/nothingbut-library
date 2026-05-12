use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::State;

use super::{api, cookie, database, downloader};
use super::downloader::SharedDownloadState;
use super::models::{
    BilibiliAccount, BilibiliDownload, BilibiliSettings, BilibiliUploader, BilibiliVideo,
    DownloadProgress, LoginStatus, QrcodeData,
};
use super::wbi::WbiSigner;

// --- 登录 ---

#[tauri::command]
pub async fn bilibili_generate_qrcode() -> Result<QrcodeData, String> {
    let client = api::build_client()?;
    api::generate_qrcode(&client).await
}

#[tauri::command]
pub async fn bilibili_poll_qrcode(
    qrcode_key: String,
    pool: State<'_, SqlitePool>,
) -> Result<LoginStatus, String> {
    let client = api::build_client()?;
    let poll = api::poll_qrcode(&client, &qrcode_key).await?;

    if poll.code == 0 {
        let (cookies, refresh_token) = api::parse_login_result(&poll)?;
        let cookies_json =
            serde_json::to_string(&cookies).map_err(|e| format!("序列化 cookie 失败: {}", e))?;

        let authed_client = api::build_authed_client(&cookies)?;
        let info = get_user_info_from_nav(&authed_client).await?;

        let account = database::upsert_account(
            &pool,
            info.uid,
            &info.username,
            info.avatar_url.as_deref(),
            &cookies_json,
            refresh_token.as_deref(),
        )
        .await?;

        let cookie_path = get_cookie_file_path();
        cookie::save_cookie_file(
            &cookies,
            account.cookie_expires_at.unwrap_or(0),
            &cookie_path,
        )?;

        return Ok(LoginStatus {
            code: 0,
            message: "登录成功".to_string(),
            account: Some(account),
        });
    }

    let message = match poll.code {
        86101 => "等待扫码",
        86090 => "已扫码，请在手机上确认",
        86038 => "二维码已过期，请重新生成",
        _ => &poll.message,
    };

    Ok(LoginStatus {
        code: poll.code,
        message: message.to_string(),
        account: None,
    })
}

#[tauri::command]
pub async fn bilibili_get_account(
    pool: State<'_, SqlitePool>,
) -> Result<Option<BilibiliAccount>, String> {
    database::get_active_account(&pool).await
}

#[tauri::command]
pub async fn bilibili_logout(pool: State<'_, SqlitePool>) -> Result<(), String> {
    let account = database::get_active_account(&pool)
        .await?
        .ok_or("没有已登录的账号")?;
    database::deactivate_account(&pool, account.uid).await?;

    let cookie_path = get_cookie_file_path();
    if cookie_path.exists() {
        std::fs::remove_file(&cookie_path).ok();
    }

    Ok(())
}

#[tauri::command]
pub async fn bilibili_check_cookie(
    pool: State<'_, SqlitePool>,
) -> Result<bool, String> {
    let account = match database::get_active_account(&pool).await? {
        Some(a) => a,
        None => return Ok(false),
    };

    let cookies = parse_cookies(&account)?;
    let client = api::build_authed_client(&cookies)?;
    let valid = api::check_cookie_valid(&client).await?;

    if !valid {
        if let Some(ref rt) = account.refresh_token {
            if let Ok(new_account) = try_refresh_cookie(&pool, &account, &cookies, rt).await {
                return Ok(new_account.is_some());
            }
        }
    }

    Ok(valid)
}

#[tauri::command]
pub async fn bilibili_refresh_cookie(
    pool: State<'_, SqlitePool>,
) -> Result<BilibiliAccount, String> {
    let account = get_active_account_or_err(&pool).await?;
    let cookies = parse_cookies(&account)?;
    let refresh_token = account
        .refresh_token
        .as_deref()
        .ok_or("没有 refresh_token，请重新扫码登录")?;

    try_refresh_cookie(&pool, &account, &cookies, refresh_token)
        .await?
        .ok_or_else(|| "刷新 cookie 失败，请重新扫码登录".to_string())
}

async fn try_refresh_cookie(
    pool: &SqlitePool,
    account: &BilibiliAccount,
    cookies: &super::models::BilibiliCookies,
    refresh_token: &str,
) -> Result<Option<BilibiliAccount>, String> {
    let client = api::build_authed_client(cookies)?;

    let refresh_csrf = match api::get_refresh_csrf(&client).await {
        Ok(csrf) => csrf,
        Err(_) => return Ok(None),
    };

    let result = match api::refresh_cookie(
        &client,
        &cookies.bili_jct,
        &refresh_csrf,
        refresh_token,
    )
    .await
    {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    api::confirm_refresh(&client, &cookies.bili_jct, refresh_token)
        .await
        .ok();

    let authed_client = api::build_authed_client(cookies)?;
    let nav_resp: serde_json::Value = authed_client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .send()
        .await
        .map_err(|e| format!("获取用户信息失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析用户信息失败: {}", e))?;

    let new_cookies_json = serde_json::to_string(cookies)
        .map_err(|e| format!("序列化 cookie 失败: {}", e))?;

    let updated = database::upsert_account(
        pool,
        account.uid,
        nav_resp["data"]["uname"].as_str().unwrap_or(&account.username),
        nav_resp["data"]["face"].as_str(),
        &new_cookies_json,
        Some(&result.new_refresh_token),
    )
    .await?;

    let cookie_path = get_cookie_file_path();
    cookie::save_cookie_file(cookies, updated.cookie_expires_at.unwrap_or(0), &cookie_path)?;

    Ok(Some(updated))
}

// --- UP 主 ---

#[tauri::command]
pub async fn bilibili_add_uploader(
    library_id: i64,
    mid: i64,
    pool: State<'_, SqlitePool>,
) -> Result<BilibiliUploader, String> {
    let account = get_active_account_or_err(&pool).await?;
    let cookies = parse_cookies(&account)?;
    let client = api::build_authed_client(&cookies)?;
    let wbi = WbiSigner::new(client.clone());

    let info = api::get_uploader_info(&client, &wbi, mid).await?;
    let page = api::fetch_video_list(&client, &wbi, mid, 1, 1, "pubdate").await?;

    database::insert_uploader(
        &pool,
        library_id,
        info.mid,
        &info.name,
        Some(&info.face),
        page.total,
    )
    .await
}

#[tauri::command]
pub async fn bilibili_list_uploaders(
    library_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<BilibiliUploader>, String> {
    database::list_uploaders(&pool, library_id).await
}

#[tauri::command]
pub async fn bilibili_remove_uploader(
    library_id: i64,
    uploader_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    database::delete_uploader(&pool, library_id, uploader_id).await
}

// --- 视频 ---

#[tauri::command]
pub async fn bilibili_sync_videos(
    library_id: i64,
    uploader_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<BilibiliVideo>, String> {
    let account = get_active_account_or_err(&pool).await?;
    let cookies = parse_cookies(&account)?;
    let client = api::build_authed_client(&cookies)?;
    let wbi = WbiSigner::new(client.clone());

    let uploader = sqlx::query_as::<_, BilibiliUploader>(
        "SELECT * FROM bilibili_uploaders WHERE id = ? AND library_id = ?",
    )
    .bind(uploader_id)
    .bind(library_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| format!("查询 UP 主失败: {}", e))?
    .ok_or("UP 主不存在")?;

    let first_page =
        api::fetch_video_list(&client, &wbi, uploader.mid, 1, 50, "pubdate").await?;
    let total_pages = (first_page.total + 49) / 50;

    for video in &first_page.videos {
        let duration = api::parse_duration_str(&video.duration_str);
        database::upsert_video(
            &pool,
            library_id,
            uploader_id,
            &video.bvid,
            Some(video.aid),
            &video.title,
            Some(&video.description),
            Some(&video.cover_url),
            Some(duration),
            video.play_count,
            Some(video.created),
        )
        .await?;
    }

    for page_num in 2..=total_pages {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let page =
            api::fetch_video_list(&client, &wbi, uploader.mid, page_num, 50, "pubdate").await?;

        for video in &page.videos {
            let duration = api::parse_duration_str(&video.duration_str);
            database::upsert_video(
                &pool,
                library_id,
                uploader_id,
                &video.bvid,
                Some(video.aid),
                &video.title,
                Some(&video.description),
                Some(&video.cover_url),
                Some(duration),
                video.play_count,
                Some(video.created),
            )
            .await?;
        }
    }

    database::update_uploader_sync_time(&pool, uploader_id, first_page.total).await?;

    database::list_videos(&pool, library_id, uploader_id).await
}

#[tauri::command]
pub async fn bilibili_list_videos(
    library_id: i64,
    uploader_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<BilibiliVideo>, String> {
    database::list_videos(&pool, library_id, uploader_id).await
}

#[tauri::command]
pub async fn bilibili_search_videos(
    library_id: i64,
    query: String,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<BilibiliVideo>, String> {
    database::search_videos(&pool, library_id, &query).await
}

// --- 下载 ---

#[tauri::command]
pub async fn bilibili_check_ytdlp() -> Result<String, String> {
    downloader::check_ytdlp_installed()
}

#[tauri::command]
pub async fn bilibili_start_download(
    library_id: i64,
    video_ids: Vec<i64>,
    download_dir: String,
    audio_format: Option<String>,
    pool: State<'_, SqlitePool>,
    download_state: State<'_, SharedDownloadState>,
) -> Result<(), String> {
    let format = audio_format.unwrap_or_else(|| "m4a".to_string());
    let cookie_path = get_cookie_file_path();

    if !cookie_path.exists() {
        return Err("请先登录 B 站".to_string());
    }

    let download_dir = PathBuf::from(&download_dir);

    for video_id in &video_ids {
        database::create_download(&pool, library_id, *video_id, &format).await?;
    }

    let pool_clone = pool.inner().clone();
    let state_clone = download_state.inner().clone();
    let video_ids_clone = video_ids.clone();
    let format_clone = format.clone();

    tokio::spawn(async move {
        for video_id in video_ids_clone {
            let video = match database::get_video_by_id(&pool_clone, video_id).await {
                Ok(Some(v)) => v,
                _ => continue,
            };

            let uploader = match sqlx::query_as::<_, BilibiliUploader>(
                "SELECT * FROM bilibili_uploaders WHERE id = ?",
            )
            .bind(video.uploader_id)
            .fetch_optional(&pool_clone)
            .await
            {
                Ok(Some(u)) => u,
                _ => continue,
            };

            let download = match database::get_download_by_video(&pool_clone, library_id, video_id)
                .await
            {
                Ok(Some(d)) => d,
                _ => continue,
            };

            database::update_download_status(&pool_clone, download.id, "downloading", None, None, None)
                .await
                .ok();

            match downloader::download_audio(
                state_clone.clone(),
                &video.bvid,
                video_id,
                &cookie_path,
                &download_dir,
                &uploader.name,
                &format_clone,
            )
            .await
            {
                Ok(path) => {
                    let file_size = std::fs::metadata(&path).map(|m| m.len() as i64).ok();
                    database::update_download_status(
                        &pool_clone,
                        download.id,
                        "completed",
                        Some(path.to_string_lossy().as_ref()),
                        file_size,
                        None,
                    )
                    .await
                    .ok();
                }
                Err(e) => {
                    database::update_download_status(
                        &pool_clone,
                        download.id,
                        "failed",
                        None,
                        None,
                        Some(&e),
                    )
                    .await
                    .ok();
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    Ok(())
}

#[tauri::command]
pub async fn bilibili_get_download_progress(
    download_state: State<'_, SharedDownloadState>,
) -> Result<Option<DownloadProgress>, String> {
    let s = download_state.lock().await;
    Ok(s.current_progress.clone())
}

#[tauri::command]
pub async fn bilibili_list_downloads(
    library_id: i64,
    pool: State<'_, SqlitePool>,
) -> Result<Vec<BilibiliDownload>, String> {
    database::list_downloads(&pool, library_id).await
}

#[tauri::command]
pub async fn bilibili_retry_download(
    library_id: i64,
    download_id: i64,
    download_dir: String,
    pool: State<'_, SqlitePool>,
    download_state: State<'_, SharedDownloadState>,
) -> Result<(), String> {
    let download = sqlx::query_as::<_, BilibiliDownload>(
        "SELECT * FROM bilibili_downloads WHERE id = ? AND library_id = ?",
    )
    .bind(download_id)
    .bind(library_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| format!("查询下载记录失败: {}", e))?
    .ok_or("下载记录不存在")?;

    bilibili_start_download(
        library_id,
        vec![download.video_id],
        download_dir,
        Some(download.audio_format),
        pool,
        download_state,
    )
    .await
}

#[tauri::command]
pub async fn bilibili_cancel_download(
    download_state: State<'_, SharedDownloadState>,
) -> Result<(), String> {
    downloader::cancel_download(&download_state).await;
    Ok(())
}

// --- 设置 ---

#[tauri::command]
pub async fn bilibili_get_settings(
    pool: State<'_, SqlitePool>,
) -> Result<BilibiliSettings, String> {
    let (installed, version) = match downloader::check_ytdlp_installed() {
        Ok(v) => (true, Some(v)),
        Err(_) => (false, None),
    };

    let download_dir = database::get_setting(&pool, "download_dir")
        .await?
        .unwrap_or_else(|| {
            get_default_download_dir().to_string_lossy().to_string()
        });

    let audio_format = database::get_setting(&pool, "audio_format")
        .await?
        .unwrap_or_else(|| "m4a".to_string());

    Ok(BilibiliSettings {
        download_dir,
        audio_format,
        ytdlp_installed: installed,
        ytdlp_version: version,
    })
}

#[tauri::command]
pub async fn bilibili_set_download_dir(
    path: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    database::set_setting(&pool, "download_dir", &path).await
}

#[tauri::command]
pub async fn bilibili_set_audio_format(
    format: String,
    pool: State<'_, SqlitePool>,
) -> Result<(), String> {
    let valid_formats = ["m4a", "mp3"];
    if !valid_formats.contains(&format.as_str()) {
        return Err(format!("不支持的音频格式: {}，可选: m4a, mp3", format));
    }
    database::set_setting(&pool, "audio_format", &format).await
}

// --- 辅助函数 ---

async fn get_active_account_or_err(pool: &SqlitePool) -> Result<BilibiliAccount, String> {
    database::get_active_account(pool)
        .await?
        .ok_or("请先登录 B 站".to_string())
}

fn parse_cookies(account: &BilibiliAccount) -> Result<super::models::BilibiliCookies, String> {
    serde_json::from_str(&account.cookies_json)
        .map_err(|e| format!("解析 cookie 失败: {}", e))
}

struct UserInfo {
    uid: i64,
    username: String,
    avatar_url: Option<String>,
}

async fn get_user_info_from_nav(client: &reqwest::Client) -> Result<UserInfo, String> {
    let resp: serde_json::Value = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .send()
        .await
        .map_err(|e| format!("获取用户信息失败: {}", e))?
        .json()
        .await
        .map_err(|e| format!("解析用户信息失败: {}", e))?;

    let data = &resp["data"];
    Ok(UserInfo {
        uid: data["mid"].as_i64().ok_or("缺少 uid")?,
        username: data["uname"].as_str().ok_or("缺少用户名")?.to_string(),
        avatar_url: data["face"].as_str().map(|s| s.to_string()),
    })
}

fn get_cookie_file_path() -> PathBuf {
    let dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.nothingbut.library");
    dir.join("bilibili_cookies.txt")
}

fn get_default_download_dir() -> PathBuf {
    dirs::download_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("BilibiliAudio")
}
