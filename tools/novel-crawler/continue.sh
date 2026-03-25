#!/bin/bash
# 小说爬虫工具 - 继续上次进度
# 此脚本将从上次停止的位置继续抓取

set -e  # Exit on error

echo "========================================"
echo "小说爬虫工具 - 继续上次进度"
echo "========================================"
echo ""

echo "[1/2] 检查当前状态..."
echo ""

python main.py stats

echo ""
echo "[2/2] 继续抓取数据（增量模式）..."
echo ""

python main.py crawl --mode incremental

if [ $? -ne 0 ]; then
    echo "❌ 抓取失败！"
    exit 1
fi

echo ""
echo "========================================"
echo "✓ 增量抓取完成！"
echo "========================================"
echo ""
