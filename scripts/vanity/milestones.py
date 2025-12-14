#!/usr/bin/env python3
"""
milestones.py - 项目里程碑成就墙

用法:
    python milestones.py [--html]

分析 git 历史，找出项目的各种里程碑时刻
"""

import subprocess
import sys
import io
from pathlib import Path
from datetime import datetime
from dataclasses import dataclass

# Windows UTF-8 输出
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

SCRIPT_DIR = Path(__file__).parent
PROJECT_DIR = SCRIPT_DIR.parent.parent


@dataclass
class Milestone:
    emoji: str
    title: str
    date: str
    detail: str
    category: str  # code, commit, time


# ANSI 颜色
class Colors:
    RESET = "\033[0m"
    BOLD = "\033[1m"
    DIM = "\033[2m"

    ROSE = "\033[38;5;211m"
    GOLD = "\033[38;5;222m"
    PINE = "\033[38;5;109m"
    FOAM = "\033[38;5;152m"
    IRIS = "\033[38;5;183m"
    TEXT = "\033[38;5;254m"
    MUTED = "\033[38;5;103m"

    # 背景
    BG_ROSE = "\033[48;5;52m"
    BG_GOLD = "\033[48;5;94m"
    BG_PINE = "\033[48;5;23m"


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


def get_loc_history() -> list[tuple[str, int]]:
    """获取代码量历史"""
    loc_file = SCRIPT_DIR.parent / "line_count" / "loc_history.csv"
    history = []
    if loc_file.exists():
        with open(loc_file, "r", encoding="utf-8") as f:
            next(f)  # 跳过标题
            for line in f:
                parts = line.strip().split(",")
                if len(parts) >= 8:
                    date = parts[0]
                    total = int(parts[7])
                    history.append((date, total))
    return history


def get_commit_history() -> list[tuple[str, str, str]]:
    """获取提交历史 [(hash, date, message), ...]"""
    output = run_git(["log", "--format=%H|%ad|%s", "--date=short", "--reverse"])
    commits = []
    for line in output.split("\n"):
        if "|" in line:
            parts = line.split("|", 2)
            if len(parts) == 3:
                commits.append((parts[0], parts[1], parts[2]))
    return commits


def find_milestones() -> list[Milestone]:
    """找出所有里程碑"""
    milestones = []

    # 代码量里程碑
    loc_history = get_loc_history()
    loc_milestones = [1000, 5000, 10000, 20000, 30000, 50000, 75000, 100000]

    for threshold in loc_milestones:
        for date, total in loc_history:
            if total >= threshold:
                emoji = {
                    1000: "🌱",
                    5000: "🌿",
                    10000: "🎯",
                    20000: "🚀",
                    30000: "⭐",
                    50000: "💫",
                    75000: "🔥",
                    100000: "👑",
                }.get(threshold, "📈")

                milestones.append(Milestone(
                    emoji=emoji,
                    title=f"{threshold // 1000}K 行代码",
                    date=date,
                    detail=f"代码量突破 {threshold:,} 行",
                    category="code"
                ))
                break

    # 提交数里程碑
    commits = get_commit_history()
    commit_milestones = [1, 10, 50, 100, 200, 300, 500, 1000]

    for threshold in commit_milestones:
        if len(commits) >= threshold:
            commit = commits[threshold - 1]
            emoji = {
                1: "🎬",
                10: "🎯",
                50: "⚡",
                100: "💯",
                200: "🏆",
                300: "🎖️",
                500: "🌟",
                1000: "👑",
            }.get(threshold, "📝")

            milestones.append(Milestone(
                emoji=emoji,
                title=f"第 {threshold} 次提交",
                date=commit[1],
                detail=commit[2][:50],
                category="commit"
            ))

    # 时间里程碑
    if commits:
        first_date = datetime.strptime(commits[0][1], "%Y-%m-%d")
        now = datetime.now()

        time_milestones = [
            (7, "🗓️", "一周年... 不对，一周"),
            (30, "📅", "满月纪念"),
            (60, "🌙", "两个月"),
            (90, "🎊", "三个月"),
            (180, "🎉", "半年纪念"),
            (365, "🎂", "一周年"),
        ]

        for days, emoji, title in time_milestones:
            if (now - first_date).days >= days:
                milestone_date = first_date.replace(
                    day=min(first_date.day, 28)  # 避免月末问题
                )
                # 简单计算里程碑日期
                from datetime import timedelta
                milestone_date = first_date + timedelta(days=days)

                milestones.append(Milestone(
                    emoji=emoji,
                    title=title,
                    date=milestone_date.strftime("%Y-%m-%d"),
                    detail=f"开发 {days} 天",
                    category="time"
                ))

    # 首次提交
    if commits:
        milestones.append(Milestone(
            emoji="🎬",
            title="项目启动",
            date=commits[0][1],
            detail=commits[0][2][:50],
            category="time"
        ))

    # 按日期排序
    milestones.sort(key=lambda m: m.date)

    return milestones


def print_milestones(milestones: list[Milestone]):
    """打印里程碑墙"""

    # Windows 终端颜色支持
    if sys.platform == "win32":
        import os
        os.system("")

    print()
    print(f"{Colors.ROSE}╔{'═' * 58}╗{Colors.RESET}")
    print(f"{Colors.ROSE}║{Colors.RESET}  {Colors.BOLD}🏆 Cutie 项目里程碑{Colors.RESET}{' ' * 36}{Colors.ROSE}║{Colors.RESET}")
    print(f"{Colors.ROSE}╠{'═' * 58}╣{Colors.RESET}")

    # 按类别分组
    by_category = {"code": [], "commit": [], "time": []}
    for m in milestones:
        by_category[m.category].append(m)

    category_names = {
        "code": ("📊 代码量成就", Colors.FOAM),
        "commit": ("📝 提交成就", Colors.GOLD),
        "time": ("⏰ 时间成就", Colors.IRIS),
    }

    for category, (name, color) in category_names.items():
        items = by_category[category]
        if not items:
            continue

        print(f"{Colors.ROSE}║{Colors.RESET}")
        print(f"{Colors.ROSE}║{Colors.RESET}  {color}{name}{Colors.RESET}")
        print(f"{Colors.ROSE}║{Colors.RESET}  {Colors.MUTED}{'─' * 40}{Colors.RESET}")

        for m in items:
            line = f"  {m.emoji}  {m.title:<16} {Colors.MUTED}{m.date}{Colors.RESET}"
            # 计算实际显示宽度
            padding = 58 - len(f"  {m.emoji}  {m.title:<16} {m.date}") - 2
            print(f"{Colors.ROSE}║{Colors.RESET}{line}{' ' * max(0, padding)}{Colors.ROSE}║{Colors.RESET}")

    print(f"{Colors.ROSE}║{Colors.RESET}")
    print(f"{Colors.ROSE}╚{'═' * 58}╝{Colors.RESET}")

    # 统计
    print()
    print(f"  {Colors.MUTED}共解锁 {Colors.TEXT}{len(milestones)}{Colors.MUTED} 个成就{Colors.RESET}")
    print()


def generate_html(milestones: list[Milestone]):
    """生成 HTML 成就墙"""

    by_category = {"code": [], "commit": [], "time": []}
    for m in milestones:
        by_category[m.category].append(m)

    def render_category(items, title, color):
        if not items:
            return ""
        html = f'<div class="category"><h2 style="color:{color}">{title}</h2><div class="badges">'
        for m in items:
            html += f'''
                <div class="badge">
                    <div class="emoji">{m.emoji}</div>
                    <div class="title">{m.title}</div>
                    <div class="date">{m.date}</div>
                    <div class="detail">{m.detail}</div>
                </div>
            '''
        html += '</div></div>'
        return html

    html = f'''<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <title>Cutie 里程碑成就墙</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            min-height: 100vh;
            padding: 40px 20px;
            color: #e0e0e0;
        }}
        .container {{ max-width: 900px; margin: 0 auto; }}
        h1 {{
            text-align: center;
            color: #eb6f92;
            margin-bottom: 40px;
            font-size: 2.5rem;
        }}
        .category {{
            margin-bottom: 40px;
        }}
        .category h2 {{
            margin-bottom: 20px;
            padding-left: 10px;
            border-left: 4px solid currentColor;
        }}
        .badges {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
            gap: 15px;
        }}
        .badge {{
            background: rgba(255, 255, 255, 0.05);
            border-radius: 16px;
            padding: 20px;
            text-align: center;
            transition: transform 0.2s, box-shadow 0.2s;
            cursor: default;
        }}
        .badge:hover {{
            transform: translateY(-5px);
            box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        }}
        .badge .emoji {{
            font-size: 2.5rem;
            margin-bottom: 10px;
        }}
        .badge .title {{
            font-weight: bold;
            color: #e0def4;
            margin-bottom: 5px;
        }}
        .badge .date {{
            color: #908caa;
            font-size: 0.85rem;
            margin-bottom: 8px;
        }}
        .badge .detail {{
            color: #6e6a86;
            font-size: 0.8rem;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
        }}
        .stats {{
            text-align: center;
            margin-top: 40px;
            padding: 20px;
            background: rgba(255, 255, 255, 0.03);
            border-radius: 12px;
        }}
        .stats .count {{
            font-size: 3rem;
            font-weight: bold;
            color: #f6c177;
        }}
        .stats .label {{
            color: #908caa;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🏆 Cutie 里程碑</h1>

        {render_category(by_category["code"], "📊 代码量成就", "#9ccfd8")}
        {render_category(by_category["commit"], "📝 提交成就", "#f6c177")}
        {render_category(by_category["time"], "⏰ 时间成就", "#c4a7e7")}

        <div class="stats">
            <div class="count">{len(milestones)}</div>
            <div class="label">成就已解锁</div>
        </div>
    </div>
</body>
</html>
'''

    output_path = SCRIPT_DIR / "milestones.html"
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(html)
    print(f"HTML 成就墙已保存到: {output_path}")
    return output_path


def main():
    milestones = find_milestones()
    print_milestones(milestones)

    if "--html" in sys.argv:
        import webbrowser
        html_path = generate_html(milestones)
        try:
            webbrowser.open(f"file://{html_path.absolute()}")
        except Exception:
            pass


if __name__ == "__main__":
    main()
