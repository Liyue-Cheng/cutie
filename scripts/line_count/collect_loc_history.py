#!/usr/bin/env python3
"""
collect_loc_history.py - 收集项目代码量历史数据（安全版本，支持增量更新）

用法:
    python collect_loc_history.py [--full]

    --full: 强制重新收集所有数据

输出: scripts/line_count/loc_history.csv

安全性: 此脚本不会修改工作区，使用 git archive 读取历史文件
"""

import subprocess
import sys
import csv
import tempfile
import shutil
from pathlib import Path

# 路径配置
SCRIPT_DIR = Path(__file__).parent
PROJECT_DIR = SCRIPT_DIR.parent.parent  # scripts/line_count -> scripts -> project root
OUTPUT_FILE = SCRIPT_DIR / "loc_history.csv"

# 要统计的目录
DIRS_TO_COUNT = ["src", "src-tauri/src"]


def run_git(args: list[str]) -> tuple[str, bool]:
    """运行git命令并返回输出和成功状态"""
    result = subprocess.run(
        ["git"] + args,
        cwd=PROJECT_DIR,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace"
    )
    return result.stdout.strip(), result.returncode == 0


def run_cloc(directory: Path) -> dict:
    """运行cloc统计代码行数"""
    if not directory.exists():
        return {"code": 0, "comment": 0, "blank": 0}

    try:
        result = subprocess.run(
            ["cloc", str(directory), "--csv", "--quiet"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace"
        )
        lines = result.stdout.strip().split("\n")
        for line in reversed(lines):
            if line and not line.startswith("files"):
                parts = line.split(",")
                if len(parts) >= 5:
                    return {
                        "blank": int(parts[2]) if parts[2].isdigit() else 0,
                        "comment": int(parts[3]) if parts[3].isdigit() else 0,
                        "code": int(parts[4]) if parts[4].isdigit() else 0,
                    }
    except Exception:
        pass

    return {"code": 0, "comment": 0, "blank": 0}


def count_lines_simple(directory: Path) -> int:
    """简单行数统计（备用）"""
    if not directory.exists():
        return 0

    total = 0
    extensions = {".ts", ".tsx", ".vue", ".js", ".jsx", ".rs", ".css", ".scss"}
    for f in directory.rglob("*"):
        if f.is_file() and f.suffix in extensions:
            try:
                with open(f, "r", encoding="utf-8", errors="ignore") as fp:
                    total += sum(1 for line in fp if line.strip())
            except Exception:
                pass
    return total


def get_all_dates() -> list[str]:
    """获取所有有commit的日期"""
    output, _ = run_git(["log", "--all", "--format=%ad", "--date=short"])
    dates = sorted(set(output.split("\n")))
    return [d for d in dates if d]


def get_last_collected_date() -> str | None:
    """获取已收集的最后日期"""
    if not OUTPUT_FILE.exists():
        return None
    with open(OUTPUT_FILE, "r", encoding="utf-8") as f:
        lines = f.readlines()
        if len(lines) > 1:
            last_line = lines[-1].strip()
            if last_line:
                return last_line.split(",")[0]
    return None


def get_commit_for_date(date: str) -> str | None:
    """获取指定日期最后一个commit"""
    output, _ = run_git([
        "log", "--all", "--format=%H",
        f"--until={date} 23:59:59", "-1"
    ])
    return output if output else None


def collect_stats_for_commit(commit: str) -> dict | None:
    """
    安全地收集指定commit的代码统计
    使用 git archive 提取到临时目录，不影响工作区
    """
    temp_dir = None
    try:
        temp_dir = Path(tempfile.mkdtemp(prefix="loc_history_"))

        # 为每个目录提取文件
        frontend_stats = {"code": 0, "comment": 0, "blank": 0}
        backend_stats = {"code": 0, "comment": 0, "blank": 0}

        # 提取前端代码
        result = subprocess.run(
            ["git", "archive", "--format=tar", commit, "--", "src"],
            cwd=PROJECT_DIR,
            capture_output=True
        )
        if result.returncode == 0 and result.stdout:
            src_dir = temp_dir / "frontend"
            src_dir.mkdir()
            subprocess.run(
                ["tar", "-xf", "-"],
                cwd=src_dir,
                input=result.stdout,
                capture_output=True
            )
            frontend_path = src_dir / "src"
            if frontend_path.exists():
                frontend_stats = run_cloc(frontend_path)

        # 提取后端代码
        result = subprocess.run(
            ["git", "archive", "--format=tar", commit, "--", "src-tauri/src"],
            cwd=PROJECT_DIR,
            capture_output=True
        )
        if result.returncode == 0 and result.stdout:
            backend_dir = temp_dir / "backend"
            backend_dir.mkdir()
            subprocess.run(
                ["tar", "-xf", "-"],
                cwd=backend_dir,
                input=result.stdout,
                capture_output=True
            )
            backend_path = backend_dir / "src-tauri" / "src"
            if backend_path.exists():
                backend_stats = run_cloc(backend_path)

        return {
            "frontend": frontend_stats,
            "backend": backend_stats,
            "total_code": frontend_stats["code"] + backend_stats["code"]
        }

    except Exception as e:
        print(f"\n错误: {e}")
        return None

    finally:
        if temp_dir and temp_dir.exists():
            shutil.rmtree(temp_dir, ignore_errors=True)


def main():
    print("=== 代码量历史收集工具 (安全版本) ===")
    print(f"项目目录: {PROJECT_DIR}")
    print(f"输出文件: {OUTPUT_FILE}")
    print("注意: 此脚本不会修改工作区")
    print()

    # 检查是否强制全量
    force_full = "--full" in sys.argv
    if force_full:
        print("强制全量模式")

    # 获取已收集的最后日期
    last_collected = None if force_full else get_last_collected_date()
    if last_collected:
        print(f"已收集到: {last_collected}")

    # 初始化CSV文件
    if not OUTPUT_FILE.exists() or force_full:
        with open(OUTPUT_FILE, "w", encoding="utf-8", newline="") as f:
            writer = csv.writer(f)
            writer.writerow([
                "date",
                "frontend_code", "frontend_comment", "frontend_blank",
                "backend_code", "backend_comment", "backend_blank",
                "total_code"
            ])
        last_collected = None

    # 获取需要收集的日期
    all_dates = get_all_dates()
    dates_to_collect = [
        d for d in all_dates
        if not last_collected or d > last_collected
    ]

    if not dates_to_collect:
        print("数据已是最新，无需更新")
        return

    print(f"需要收集 {len(dates_to_collect)} 天的数据")
    print()

    # 收集每天的数据
    with open(OUTPUT_FILE, "a", encoding="utf-8", newline="") as f:
        writer = csv.writer(f)

        for i, date in enumerate(dates_to_collect, 1):
            print(f"\r[{i}/{len(dates_to_collect)}] 正在处理 {date} ...", end="", flush=True)

            commit = get_commit_for_date(date)
            if not commit:
                continue

            stats = collect_stats_for_commit(commit)
            if not stats:
                continue

            writer.writerow([
                date,
                stats["frontend"]["code"],
                stats["frontend"]["comment"],
                stats["frontend"]["blank"],
                stats["backend"]["code"],
                stats["backend"]["comment"],
                stats["backend"]["blank"],
                stats["total_code"]
            ])
            f.flush()

    print()
    print()
    print("完成！")
    print(f"数据已保存到: {OUTPUT_FILE}")
    print()
    print("运行以下命令生成图表:")
    print(f"  python {SCRIPT_DIR / 'generate_loc_chart.py'}")


if __name__ == "__main__":
    main()
