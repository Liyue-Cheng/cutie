#!/usr/bin/env python3
"""
commit_time_heatmap.py - 提交时间分布分析

用法:
    python commit_time_heatmap.py [--html]

分析 git 提交的时间分布，展示你的"肝度"
"""

import subprocess
import sys
import io
from pathlib import Path
from collections import defaultdict
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

    # 热力图颜色 (从浅到深)
    HEAT = [
        "\033[48;5;234m",   # 最浅 (几乎没有)
        "\033[48;5;22m",    # 深绿
        "\033[48;5;28m",
        "\033[48;5;34m",
        "\033[48;5;40m",    # 最深 (最活跃)
    ]

    ROSE = "\033[38;5;211m"
    FOAM = "\033[38;5;152m"
    TEXT = "\033[38;5;254m"
    MUTED = "\033[38;5;103m"


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


def get_commit_times() -> list[datetime]:
    """获取所有提交的时间"""
    output = run_git(["log", "--format=%aI"])  # ISO 8601 格式
    times = []
    for line in output.split("\n"):
        if line:
            try:
                # 解析 ISO 格式时间
                dt = datetime.fromisoformat(line.replace("Z", "+00:00"))
                times.append(dt)
            except ValueError:
                pass
    return times


def analyze_by_hour(times: list[datetime]) -> dict[int, int]:
    """按小时统计"""
    by_hour = defaultdict(int)
    for dt in times:
        by_hour[dt.hour] += 1
    return dict(by_hour)


def analyze_by_weekday(times: list[datetime]) -> dict[int, int]:
    """按星期几统计"""
    by_weekday = defaultdict(int)
    for dt in times:
        by_weekday[dt.weekday()] += 1
    return dict(by_weekday)


def analyze_by_weekday_hour(times: list[datetime]) -> dict[tuple[int, int], int]:
    """按星期几+小时统计 (用于热力图)"""
    by_wh = defaultdict(int)
    for dt in times:
        by_wh[(dt.weekday(), dt.hour)] += 1
    return dict(by_wh)


def get_heat_color(value: int, max_value: int) -> str:
    """获取热力图颜色"""
    if max_value == 0:
        return Colors.HEAT[0]

    ratio = value / max_value
    if ratio == 0:
        idx = 0
    elif ratio < 0.25:
        idx = 1
    elif ratio < 0.5:
        idx = 2
    elif ratio < 0.75:
        idx = 3
    else:
        idx = 4

    return Colors.HEAT[idx]


def print_hour_chart(by_hour: dict[int, int]):
    """打印小时分布柱状图"""
    max_count = max(by_hour.values()) if by_hour else 1
    bar_width = 40

    print(f"\n{Colors.ROSE}  ⏰ 提交时间分布 (按小时){Colors.RESET}\n")

    for hour in range(24):
        count = by_hour.get(hour, 0)
        bar_len = int((count / max_count) * bar_width) if max_count > 0 else 0

        # 时间标签
        label = f"  {hour:02d}:00"

        # 柱状图
        bar = "█" * bar_len

        # 根据时间段选择颜色
        if 0 <= hour < 6:
            color = "\033[38;5;103m"  # 凌晨 - 暗淡
        elif 6 <= hour < 9:
            color = "\033[38;5;222m"  # 早晨 - 金色
        elif 9 <= hour < 18:
            color = "\033[38;5;152m"  # 工作时间 - 青色
        elif 18 <= hour < 22:
            color = "\033[38;5;211m"  # 晚上 - 玫红
        else:
            color = "\033[38;5;183m"  # 深夜 - 紫色

        count_str = f" {count}" if count > 0 else ""
        print(f"{label} {color}{bar}{Colors.RESET}{Colors.MUTED}{count_str}{Colors.RESET}")

    print()


def print_weekday_chart(by_weekday: dict[int, int]):
    """打印星期分布"""
    weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
    max_count = max(by_weekday.values()) if by_weekday else 1
    bar_width = 30

    print(f"\n{Colors.ROSE}  📅 提交分布 (按星期){Colors.RESET}\n")

    for day in range(7):
        count = by_weekday.get(day, 0)
        bar_len = int((count / max_count) * bar_width) if max_count > 0 else 0

        label = f"  {weekdays[day]}"
        bar = "█" * bar_len

        # 周末用不同颜色
        color = "\033[38;5;211m" if day >= 5 else "\033[38;5;152m"

        count_str = f" {count}" if count > 0 else ""
        print(f"{label} {color}{bar}{Colors.RESET}{Colors.MUTED}{count_str}{Colors.RESET}")

    print()


def print_heatmap(by_wh: dict[tuple[int, int], int]):
    """打印热力图"""
    weekdays = ["一", "二", "三", "四", "五", "六", "日"]
    max_count = max(by_wh.values()) if by_wh else 1

    print(f"\n{Colors.ROSE}  🔥 活跃热力图{Colors.RESET}\n")

    # 小时标签
    hour_label = "      "
    for h in range(0, 24, 3):
        hour_label += f"{h:02d}    "
    print(f"{Colors.MUTED}{hour_label}{Colors.RESET}")

    # 热力图
    for day in range(7):
        row = f"  {Colors.MUTED}{weekdays[day]}{Colors.RESET}  "
        for hour in range(24):
            count = by_wh.get((day, hour), 0)
            color = get_heat_color(count, max_count)
            row += f"{color}  {Colors.RESET}"
        print(row)

    # 图例
    print()
    legend = f"  {Colors.MUTED}少{Colors.RESET} "
    for color in Colors.HEAT:
        legend += f"{color}  {Colors.RESET}"
    legend += f" {Colors.MUTED}多{Colors.RESET}"
    print(legend)
    print()


def print_summary(times: list[datetime], by_hour: dict[int, int], by_weekday: dict[int, int]):
    """打印统计摘要"""
    print(f"\n{Colors.ROSE}  📊 统计摘要{Colors.RESET}\n")

    total = len(times)
    print(f"  总提交数: {Colors.TEXT}{total}{Colors.RESET}")

    # 最活跃的小时
    if by_hour:
        peak_hour = max(by_hour.keys(), key=lambda h: by_hour[h])
        print(f"  最活跃时段: {Colors.TEXT}{peak_hour:02d}:00 - {peak_hour:02d}:59{Colors.RESET} ({by_hour[peak_hour]} 次提交)")

    # 最活跃的星期
    weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
    if by_weekday:
        peak_day = max(by_weekday.keys(), key=lambda d: by_weekday[d])
        print(f"  最活跃日: {Colors.TEXT}{weekdays[peak_day]}{Colors.RESET} ({by_weekday[peak_day]} 次提交)")

    # 深夜肝帝指数 (0-6点的提交比例)
    late_night = sum(by_hour.get(h, 0) for h in range(0, 6))
    if total > 0:
        liver_index = (late_night / total) * 100
        if liver_index > 20:
            liver_comment = "🔥 肝帝级别!"
        elif liver_index > 10:
            liver_comment = "⚠️  注意休息"
        elif liver_index > 5:
            liver_comment = "🌙 偶尔熬夜"
        else:
            liver_comment = "😴 作息健康"
        print(f"  深夜指数: {Colors.TEXT}{liver_index:.1f}%{Colors.RESET} {liver_comment}")

    # 周末工作狂指数
    weekend = sum(by_weekday.get(d, 0) for d in [5, 6])
    if total > 0:
        weekend_ratio = (weekend / total) * 100
        print(f"  周末占比: {Colors.TEXT}{weekend_ratio:.1f}%{Colors.RESET}")

    print()


def generate_html(times: list[datetime], by_hour: dict, by_weekday: dict, by_wh: dict):
    """生成 HTML 版本"""
    weekdays = ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
    max_wh = max(by_wh.values()) if by_wh else 1

    # 热力图数据
    heatmap_data = []
    for day in range(7):
        for hour in range(24):
            count = by_wh.get((day, hour), 0)
            heatmap_data.append({"day": day, "hour": hour, "count": count})

    # 小时数据
    hour_data = [by_hour.get(h, 0) for h in range(24)]

    # 星期数据
    weekday_data = [by_weekday.get(d, 0) for d in range(7)]

    html = f'''<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <title>Cutie 提交时间分析</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            min-height: 100vh;
            padding: 20px;
            color: #e0e0e0;
        }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        h1 {{
            text-align: center;
            color: #eb6f92;
            margin-bottom: 30px;
        }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
            gap: 20px;
        }}
        .card {{
            background: rgba(255, 255, 255, 0.05);
            border-radius: 16px;
            padding: 20px;
        }}
        .card h2 {{
            color: #9ccfd8;
            margin-bottom: 15px;
            font-size: 1.1rem;
        }}
        .heatmap {{
            display: grid;
            grid-template-columns: 30px repeat(24, 1fr);
            gap: 2px;
        }}
        .heatmap-cell {{
            aspect-ratio: 1;
            border-radius: 3px;
            min-height: 15px;
        }}
        .heatmap-label {{
            display: flex;
            align-items: center;
            justify-content: center;
            color: #908caa;
            font-size: 12px;
        }}
        .hour-label {{
            font-size: 10px;
            color: #6e6a86;
            text-align: center;
        }}
        .legend {{
            display: flex;
            justify-content: center;
            align-items: center;
            gap: 5px;
            margin-top: 15px;
            font-size: 12px;
            color: #908caa;
        }}
        .legend-cell {{
            width: 15px;
            height: 15px;
            border-radius: 3px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🕐 提交时间分析</h1>

        <div class="grid">
            <div class="card">
                <h2>📊 按小时分布</h2>
                <canvas id="hourChart"></canvas>
            </div>

            <div class="card">
                <h2>📅 按星期分布</h2>
                <canvas id="weekdayChart"></canvas>
            </div>

            <div class="card" style="grid-column: 1 / -1;">
                <h2>🔥 活跃热力图</h2>
                <div id="heatmap"></div>
            </div>
        </div>
    </div>

    <script>
        // 小时分布图
        new Chart(document.getElementById('hourChart'), {{
            type: 'bar',
            data: {{
                labels: Array.from({{length: 24}}, (_, i) => i + ':00'),
                datasets: [{{
                    label: '提交次数',
                    data: {hour_data},
                    backgroundColor: 'rgba(156, 207, 216, 0.6)',
                    borderColor: '#9ccfd8',
                    borderWidth: 1
                }}]
            }},
            options: {{
                responsive: true,
                plugins: {{ legend: {{ display: false }} }},
                scales: {{
                    y: {{ beginAtZero: true, ticks: {{ color: '#908caa' }}, grid: {{ color: 'rgba(255,255,255,0.05)' }} }},
                    x: {{ ticks: {{ color: '#908caa' }}, grid: {{ display: false }} }}
                }}
            }}
        }});

        // 星期分布图
        new Chart(document.getElementById('weekdayChart'), {{
            type: 'bar',
            data: {{
                labels: {weekdays},
                datasets: [{{
                    label: '提交次数',
                    data: {weekday_data},
                    backgroundColor: {['"rgba(235, 111, 146, 0.6)"' if i >= 5 else '"rgba(156, 207, 216, 0.6)"' for i in range(7)]},
                    borderColor: {['"#eb6f92"' if i >= 5 else '"#9ccfd8"' for i in range(7)]},
                    borderWidth: 1
                }}]
            }},
            options: {{
                responsive: true,
                plugins: {{ legend: {{ display: false }} }},
                scales: {{
                    y: {{ beginAtZero: true, ticks: {{ color: '#908caa' }}, grid: {{ color: 'rgba(255,255,255,0.05)' }} }},
                    x: {{ ticks: {{ color: '#908caa' }}, grid: {{ display: false }} }}
                }}
            }}
        }});

        // 热力图
        const heatmapData = {heatmap_data};
        const maxCount = {max_wh};
        const weekdayLabels = {weekdays};

        function getHeatColor(count) {{
            if (count === 0) return '#1a1a2e';
            const ratio = count / maxCount;
            if (ratio < 0.25) return '#1e3a2f';
            if (ratio < 0.5) return '#2d5a3f';
            if (ratio < 0.75) return '#3d7a4f';
            return '#4daa5f';
        }}

        let heatmapHtml = '<div class="heatmap">';
        // 小时标签行
        heatmapHtml += '<div></div>';
        for (let h = 0; h < 24; h++) {{
            heatmapHtml += `<div class="hour-label">${{h}}</div>`;
        }}
        // 数据行
        for (let d = 0; d < 7; d++) {{
            heatmapHtml += `<div class="heatmap-label">${{weekdayLabels[d]}}</div>`;
            for (let h = 0; h < 24; h++) {{
                const item = heatmapData.find(x => x.day === d && x.hour === h);
                const count = item ? item.count : 0;
                const color = getHeatColor(count);
                heatmapHtml += `<div class="heatmap-cell" style="background:${{color}}" title="${{weekdayLabels[d]}} ${{h}}:00 - ${{count}}次"></div>`;
            }}
        }}
        heatmapHtml += '</div>';
        heatmapHtml += '<div class="legend"><span>少</span>';
        ['#1a1a2e', '#1e3a2f', '#2d5a3f', '#3d7a4f', '#4daa5f'].forEach(c => {{
            heatmapHtml += `<div class="legend-cell" style="background:${{c}}"></div>`;
        }});
        heatmapHtml += '<span>多</span></div>';
        document.getElementById('heatmap').innerHTML = heatmapHtml;
    </script>
</body>
</html>
'''
    output_path = SCRIPT_DIR / "commit_time_heatmap.html"
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(html)
    print(f"HTML 图表已保存到: {output_path}")
    return output_path


def main():
    # Windows 终端颜色支持
    if sys.platform == "win32":
        import os
        os.system("")

    print(f"\n{Colors.ROSE}{'=' * 50}{Colors.RESET}")
    print(f"{Colors.ROSE}  Cutie 提交时间分析{Colors.RESET}")
    print(f"{Colors.ROSE}{'=' * 50}{Colors.RESET}")

    # 获取数据
    times = get_commit_times()
    if not times:
        print("没有找到提交记录")
        return

    by_hour = analyze_by_hour(times)
    by_weekday = analyze_by_weekday(times)
    by_wh = analyze_by_weekday_hour(times)

    # 打印终端版本
    print_hour_chart(by_hour)
    print_weekday_chart(by_weekday)
    print_heatmap(by_wh)
    print_summary(times, by_hour, by_weekday)

    # 生成 HTML 版本
    if "--html" in sys.argv:
        import webbrowser
        html_path = generate_html(times, by_hour, by_weekday, by_wh)
        try:
            webbrowser.open(f"file://{html_path.absolute()}")
        except Exception:
            pass


if __name__ == "__main__":
    main()
