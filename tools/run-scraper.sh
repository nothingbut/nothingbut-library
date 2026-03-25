#!/bin/bash
# 小说爬虫运行脚本
# 使用方法: ./run-scraper.sh <来源站点> <书名> [作者]

echo "========================================"
echo "小说爬虫工具"
echo "========================================"
echo ""

if [ -z "$1" ]; then
    echo "用法: ./run-scraper.sh <来源站点> <书名> [作者]"
    echo ""
    echo "参数:"
    echo "  来源站点  起点 或 独阅读"
    echo "  书名      要搜索的书名"
    echo "  作者      可选，用于精确匹配"
    echo ""
    echo "示例:"
    echo "  ./run-scraper.sh 起点 '诡秘之主'"
    echo "  ./run-scraper.sh 起点 '诡秘之主' '爱潜水的乌贼'"
    echo ""
    exit 1
fi

SOURCE_SITE="$1"
TITLE="$2"
AUTHOR="${3:-}"

echo "来源站点: $SOURCE_SITE"
echo "书名: $TITLE"
if [ -n "$AUTHOR" ]; then
    echo "作者: $AUTHOR"
fi
echo ""

cd "$(dirname "$0")/../src-tauri" || exit 1

echo "正在构建并运行爬虫 CLI 工具..."
echo ""

cargo run --bin scraper_cli -- "$SOURCE_SITE" "$TITLE" $AUTHOR

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ 运行失败！"
    echo ""
    exit 1
fi
