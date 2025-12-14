#!/usr/bin/env python3
"""
project_neofetch.py - 项目信息卡片 (类似 neofetch 风格)

用法:
    python project_neofetch.py [--no-color]

在终端显示项目的各种统计信息
"""

import subprocess
import sys
import io
from pathlib import Path
from datetime import datetime

# Windows UTF-8 输出
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

SCRIPT_DIR = Path(__file__).parent
PROJECT_DIR = SCRIPT_DIR.parent.parent


# ANSI 颜色
class Colors:
    RESET = "\033[0m"
    BOLD = "\033[1m"
    DIM = "\033[2m"

    # Rose Pine 配色
    ROSE = "\033[38;5;211m"      # #eb6f92
    GOLD = "\033[38;5;222m"      # #f6c177
    PINE = "\033[38;5;109m"      # #31748f
    FOAM = "\033[38;5;152m"      # #9ccfd8
    IRIS = "\033[38;5;183m"      # #c4a7e7
    TEXT = "\033[38;5;254m"      # #e0def4
    MUTED = "\033[38;5;103m"     # #6e6a86
    SUBTLE = "\033[38;5;145m"    # #908caa


NO_COLOR = False


def c(color: str, text: str) -> str:
    """应用颜色"""
    if NO_COLOR:
        return text
    return f"{color}{text}{Colors.RESET}"


def run_git(args: list[str]) -> str:
    """运行 git 命令"""
    result = subprocess.run(
        ["git"] + args,
        cwd=PROJECT_DIR,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace"
    )
    return result.stdout.strip()


def get_project_stats():
    """获取项目统计信息"""
    stats = {}

    # 项目名
    stats["name"] = PROJECT_DIR.name.capitalize()

    # 第一次提交日期
    first_commit = run_git(["log", "--reverse", "--format=%ad", "--date=short", "-1"])
    stats["start_date"] = first_commit

    # 开发天数
    if first_commit:
        start = datetime.strptime(first_commit, "%Y-%m-%d")
        days = (datetime.now() - start).days
        stats["days"] = days
    else:
        stats["days"] = 0

    # 总提交数
    commit_count = run_git(["rev-list", "--count", "HEAD"])
    stats["commits"] = int(commit_count) if commit_count.isdigit() else 0

    # 当前分支
    branch = run_git(["branch", "--show-current"])
    stats["branch"] = branch or "unknown"

    # 代码行数 (从 loc_history.csv 读取最新数据)
    loc_file = SCRIPT_DIR.parent / "line_count" / "loc_history.csv"
    if loc_file.exists():
        with open(loc_file, "r", encoding="utf-8") as f:
            lines = f.readlines()
            if len(lines) > 1:
                last_line = lines[-1].strip().split(",")
                stats["frontend_loc"] = int(last_line[1])
                stats["backend_loc"] = int(last_line[4])
                stats["total_loc"] = int(last_line[7])
    else:
        stats["frontend_loc"] = 0
        stats["backend_loc"] = 0
        stats["total_loc"] = 0

    # 日均代码量
    if stats["days"] > 0:
        stats["daily_loc"] = stats["total_loc"] // stats["days"]
    else:
        stats["daily_loc"] = 0

    # 文件数量
    file_count = run_git(["ls-files"])
    stats["files"] = len(file_count.split("\n")) if file_count else 0

    # 最近提交
    last_commit = run_git(["log", "-1", "--format=%s"])
    stats["last_commit"] = last_commit[:40] + "..." if len(last_commit) > 40 else last_commit

    # 技术栈
    stats["frontend_stack"] = "Vue 3 + TypeScript + Vite"
    stats["backend_stack"] = "Rust + Tauri + SQLite"

    return stats


def format_number(n: int) -> str:
    """格式化数字，添加千分位"""
    return f"{n:,}"


def print_card(stats: dict):
    """打印信息卡片"""

    # ASCII Art Logo (简化版)
    logo = [
        "   ██████╗██╗   ██╗████████╗██╗███████╗",
        "  ██╔════╝██║   ██║╚══██╔══╝██║██╔════╝",
        "  ██║     ██║   ██║   ██║   ██║█████╗  ",
        "  ██║     ██║   ██║   ██║   ██║██╔══╝  ",
        "  ╚██████╗╚██████╔╝   ██║   ██║███████╗",
        "   ╚═════╝ ╚═════╝    ╚═╝   ╚═╝╚══════╝",
    ]

    # 信息行
    info_lines = [
        "",
        f"{c(Colors.ROSE, stats['name'])} {c(Colors.MUTED, '@')} {c(Colors.FOAM, stats['branch'])}",
        c(Colors.MUTED, "─" * 32),
        f"{c(Colors.FOAM, '代码行数')}  {c(Colors.TEXT, format_number(stats['total_loc']))}",
        f"{c(Colors.FOAM, '前端代码')}  {c(Colors.TEXT, format_number(stats['frontend_loc']))}",
        f"{c(Colors.FOAM, '后端代码')}  {c(Colors.TEXT, format_number(stats['backend_loc']))}",
        f"{c(Colors.FOAM, '提交次数')}  {c(Colors.TEXT, format_number(stats['commits']))}",
        f"{c(Colors.FOAM, '开发天数')}  {c(Colors.TEXT, str(stats['days']))} 天",
        f"{c(Colors.FOAM, '日均代码')}  {c(Colors.GOLD, '+' + format_number(stats['daily_loc']))} 行",
        c(Colors.MUTED, "─" * 32),
        f"{c(Colors.IRIS, '前端')}  {c(Colors.SUBTLE, stats['frontend_stack'])}",
        f"{c(Colors.IRIS, '后端')}  {c(Colors.SUBTLE, stats['backend_stack'])}",
        c(Colors.MUTED, "─" * 32),
        f"{c(Colors.MUTED, '最近')}  {c(Colors.DIM, stats['last_commit'])}",
    ]

    # 合并输出
    print()
    max_logo_width = max(len(line) for line in logo) if logo else 0

    for i in range(max(len(logo), len(info_lines))):
        logo_part = ""
        if i < len(logo):
            logo_part = c(Colors.ROSE, logo[i])
            # 计算实际显示宽度（不含 ANSI 码）
            padding = max_logo_width - len(logo[i]) + 4
        else:
            padding = max_logo_width + 4

        info_part = ""
        if i < len(info_lines):
            info_part = info_lines[i]

        if i < len(logo):
            print(f"  {logo_part}{' ' * padding}{info_part}")
        else:
            print(f"  {' ' * max_logo_width}{' ' * 4}{info_part}")

    print()

    # 颜色条
    if not NO_COLOR:
        colors = [
            "\033[41m", "\033[43m", "\033[42m", "\033[46m",
            "\033[44m", "\033[45m", "\033[47m", "\033[100m"
        ]
        color_bar = "  " + " " * max_logo_width + "    "
        for color in colors:
            color_bar += f"{color}   {Colors.RESET}"
        print(color_bar)
        print()


def main():
    global NO_COLOR

    if "--no-color" in sys.argv or "--no-colour" in sys.argv:
        NO_COLOR = True

    # Windows 终端颜色支持
    if sys.platform == "win32":
        import os
        os.system("")  # 启用 ANSI 支持

    stats = get_project_stats()
    print_card(stats)


if __name__ == "__main__":
    main()
