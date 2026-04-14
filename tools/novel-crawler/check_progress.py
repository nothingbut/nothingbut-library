"""
检查爬取进度：最后成功ID + 间隔>=100的ID对。

用法: python check_progress.py [--gap N]
  --gap N   间隔阈值，默认100
"""
import sqlite3
import sys
from pathlib import Path

DB_PATH = Path(__file__).resolve().parent / "data" / "youshu.db"
DEFAULT_GAP = 100


def check_progress(gap: int = DEFAULT_GAP) -> None:
    if not DB_PATH.exists():
        print(f"数据库不存在: {DB_PATH}")
        return

    conn = sqlite3.connect(DB_PATH)
    cur = conn.cursor()

    # 最后成功ID
    cur.execute("SELECT MAX(id) FROM books")
    row = cur.fetchone()
    last_id = row[0] if row and row[0] is not None else None

    # 总数
    cur.execute("SELECT COUNT(*) FROM books")
    total = cur.fetchone()[0]

    print(f"总书籍数: {total}")
    print(f"最后成功ID: {last_id if last_id is not None else '无数据'}")

    if last_id is None:
        conn.close()
        return

    # 查找间隔 >= gap 的ID对
    # 用窗口函数比较相邻行的id差值
    cur.execute(f"""
        SELECT prev_id, curr_id, curr_id - prev_id AS diff
        FROM (
            SELECT id AS curr_id,
                   LAG(id) OVER (ORDER BY id) AS prev_id
            FROM books
        )
        WHERE prev_id IS NOT NULL AND curr_id - prev_id >= ?
        ORDER BY prev_id
    """, (gap,))

    gaps = cur.fetchall()

    if gaps:
        print(f"\n间隔 >= {gap} 的ID对 (共 {len(gaps)} 处):")
        print(f"{'从':>8}  {'到':>8}  {'间隔':>6}")
        print("-" * 30)
        for prev_id, curr_id, diff in gaps:
            print(f"{prev_id:>8}  {curr_id:>8}  {diff:>6}")
    else:
        print(f"\n未发现间隔 >= {gap} 的ID对")

    conn.close()


if __name__ == "__main__":
    gap = DEFAULT_GAP
    for i, arg in enumerate(sys.argv[1:], 1):
        if arg == "--gap" and i < len(sys.argv) - 1:
            gap = int(sys.argv[i + 1])

    check_progress(gap)
