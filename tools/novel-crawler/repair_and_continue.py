"""
Repair gaps in crawled data.

Scan [start, end] for adjacent book ID gaps >= threshold,
then re-crawl each missing interval via main.py refresh.

Usage: python repair_and_continue.py --start START --end END [--gap GAP]
"""
import argparse
import sqlite3
import subprocess
import sys
import time
from pathlib import Path

DB_PATH = Path(__file__).resolve().parent / "data" / "youshu.db"
DEFAULT_GAP = 100


def find_gaps(start: int, end: int, gap: int) -> list[tuple[int, int, int]]:
    """Find ID gaps >= threshold between adjacent existing books in [start, end]."""
    conn = sqlite3.connect(DB_PATH)
    cur = conn.cursor()

    cur.execute(
        "SELECT id FROM books WHERE id >= ? AND id <= ? ORDER BY id",
        (start, end),
    )
    ids = [row[0] for row in cur.fetchall()]
    conn.close()

    if len(ids) < 2:
        return []

    gaps = []
    for i in range(1, len(ids)):
        diff = ids[i] - ids[i - 1]
        if diff >= gap:
            gaps.append((ids[i - 1], ids[i], diff))

    return gaps


def repair_gaps(start: int, end: int, gap: int) -> int:
    """Find and repair gaps in crawled data."""
    if not DB_PATH.exists():
        print(f"Database not found: {DB_PATH}")
        return 1

    print(f"Scanning [{start}, {end}] for gaps >= {gap}...")
    gaps = find_gaps(start, end, gap)

    if not gaps:
        print("No gaps found. All done!")
        return 0

    total_missing = sum(d - 1 for _, _, d in gaps)
    print(f"Found {len(gaps)} gaps, ~{total_missing} missing IDs")
    print()
    print(f"{'#':>4}  {'From':>8}  {'To':>8}  {'Gap':>6}  {'Missing':>8}")
    print("-" * 45)
    for i, (prev_id, curr_id, diff) in enumerate(gaps, 1):
        print(f"{i:>4}  {prev_id:>8}  {curr_id:>8}  {diff:>6}  {diff - 1:>8}")

    print()
    print(f"Starting repair of {len(gaps)} gaps...")
    print("=" * 60)

    ok_count = 0
    fail_count = 0
    script_dir = str(Path(__file__).resolve().parent)

    for i, (prev_id, curr_id, diff) in enumerate(gaps, 1):
        gap_start = prev_id + 1
        gap_end = curr_id - 1
        missing = diff - 1

        print()
        print(f"[{i}/{len(gaps)}] Gap {prev_id} -> {curr_id} ({missing} missing IDs)")
        print(f"  Range: [{gap_start}, {gap_end}]")

        t0 = time.time()
        result = subprocess.run(
            [
                sys.executable, "main.py", "refresh",
                "--start", str(gap_start),
                "--end", str(gap_end),
            ],
            cwd=script_dir,
        )
        elapsed = time.time() - t0

        if result.returncode == 0:
            print(f"  Gap {i} completed ({elapsed:.0f}s)")
            ok_count += 1
        else:
            print(f"  Gap {i} FAILED ({elapsed:.0f}s)")
            fail_count += 1

    print()
    print("=" * 60)
    print("REPAIR SUMMARY")
    print(f"  Total gaps:   {len(gaps)}")
    print(f"  Successful:   {ok_count}")
    print(f"  Failed:       {fail_count}")
    print("=" * 60)

    return 0 if fail_count == 0 else 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Repair gaps in crawled data")
    parser.add_argument("--start", type=int, required=True, help="Start ID")
    parser.add_argument("--end", type=int, required=True, help="End ID")
    parser.add_argument("--gap", type=int, default=DEFAULT_GAP,
                        help=f"Gap threshold (default: {DEFAULT_GAP})")

    args = parser.parse_args()
    sys.exit(repair_gaps(args.start, args.end, args.gap))
