use std::env;
use std::path::PathBuf;

// 复用爬虫模块
use tauri_app_lib::modules::novel::scraper;

#[tokio::main]
async fn main() {
    println!("🕷️  小说爬虫工具 v1.0");
    println!("===================\n");

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage();
        return;
    }

    let source_site = &args[1];
    let title = &args[2];
    let author = if args.len() > 3 { Some(args[3].as_str()) } else { None };
    let output_file = if args.len() > 4 { Some(PathBuf::from(&args[4])) } else { None };

    println!("📚 来源站点: {}", source_site);
    println!("📖 书名: {}", title);
    if let Some(auth) = author {
        println!("✍️  作者: {}", auth);
    }
    println!();

    match scrape(source_site, title, author, output_file).await {
        Ok(metadata) => {
            println!("✅ 爬取成功！");
            println!("\n📋 元数据:");

            if let Some(author) = metadata.author {
                println!("  ✍️  作者: {}", author);
            }

            if let Some(category) = metadata.category {
                println!("  📂 分类: {}", category);
            }

            if let Some(description) = metadata.description {
                let preview = if description.len() > 200 {
                    format!("{}...", &description[..200])
                } else {
                    description.clone()
                };
                println!("  📝 简介:\n     {}", preview);
            }

            if let Some(cover_url) = metadata.cover_url {
                println!("  🖼️  封面: {}", cover_url);
            }
        }
        Err(e) => {
            eprintln!("❌ 爬取失败: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("用法:");
    println!("  cargo run --bin scraper_cli -- <来源站点> <书名> [作者] [输出文件]");
    println!();
    println!("参数:");
    println!("  来源站点  起点 或 独阅读");
    println!("  书名      要搜索的书名或起点URL");
    println!("  作者      可选，用于精确匹配");
    println!("  输出文件  可选，将结果保存为JSON文件");
    println!();
    println!("示例:");
    println!("  cargo run --bin scraper_cli -- 起点 \"诡秘之主\"");
    println!("  cargo run --bin scraper_cli -- 起点 \"诡秘之主\" \"爱潜水的乌贼\"");
    println!("  cargo run --bin scraper_cli -- 起点 \"https://www.qidian.com/book/1010400217\"");
    println!("  cargo run --bin scraper_cli -- 起点 \"诡秘之主\" \"爱潜水的乌贼\" output.json");
    println!("  cargo run --bin scraper_cli -- 独阅读 \"某部小说\"");
}

async fn scrape(
    source_site: &str,
    title: &str,
    author: Option<&str>,
    output_file: Option<PathBuf>,
) -> Result<scraper::ScrapedMetadata, Box<dyn std::error::Error>> {
    println!("🌐 开始爬取...");

    let metadata = scraper::scrape_metadata(source_site, title, author).await?;

    // 如果指定了输出文件，保存为 JSON
    if let Some(output_path) = output_file {
        use serde_json::json;

        let output = json!({
            "source_site": source_site,
            "title": title,
            "author": metadata.author,
            "category": metadata.category,
            "description": metadata.description,
            "cover_url": metadata.cover_url,
        });

        tokio::fs::write(&output_path, serde_json::to_string_pretty(&output)?)
            .await
            .map_err(|e| format!("Failed to write output file: {}", e))?;

        println!("💾 结果已保存到: {}", output_path.display());
    }

    Ok(metadata)
}
