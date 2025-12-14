#!/usr/bin/env python3
"""
gource_video.py - 生成 Git 历史动画视频

用法:
    python gource_video.py [--output video.mp4] [--resolution 1920x1080]

依赖:
    - gource: https://gource.io/
    - ffmpeg: https://ffmpeg.org/

安装 (Windows):
    scoop install gource ffmpeg
    # 或 choco install gource ffmpeg
"""

import subprocess
import sys
import io
import shutil
from pathlib import Path
from datetime import datetime

# Windows UTF-8 输出
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

SCRIPT_DIR = Path(__file__).parent
PROJECT_DIR = SCRIPT_DIR.parent.parent
DEFAULT_OUTPUT = SCRIPT_DIR / f"cutie_history_{datetime.now().strftime('%Y%m%d')}.mp4"


def check_dependencies():
    """检查依赖是否安装"""
    missing = []
    if not shutil.which("gource"):
        missing.append("gource")
    if not shutil.which("ffmpeg"):
        missing.append("ffmpeg")

    if missing:
        print("缺少依赖:")
        for dep in missing:
            print(f"  - {dep}")
        print("\n安装方法 (Windows):")
        print("  scoop install gource ffmpeg")
        print("  # 或")
        print("  choco install gource ffmpeg")
        return False
    return True


def generate_video(output_path: Path, resolution: str = "1920x1080"):
    """生成 Gource 视频"""
    width, height = resolution.split("x")

    # Gource 配置
    gource_cmd = [
        "gource",
        str(PROJECT_DIR),
        f"-{resolution}",
        "--seconds-per-day", "0.3",        # 每天 0.3 秒
        "--auto-skip-seconds", "0.5",       # 跳过空闲
        "--file-idle-time", "0",            # 文件不消失
        "--max-file-lag", "0.1",            # 文件出现延迟
        "--hide", "filenames,dirnames",     # 隐藏文件名（太多了）
        "--font-size", "18",
        "--title", "Cutie - Development History",
        "--key",                            # 显示图例
        "--highlight-users",                # 高亮用户
        "--multi-sampling",                 # 抗锯齿
        "--stop-at-end",                    # 结束时停止
        "--output-ppm-stream", "-",         # 输出到管道
        "--output-framerate", "60",
    ]

    # FFmpeg 配置
    ffmpeg_cmd = [
        "ffmpeg",
        "-y",                               # 覆盖输出
        "-r", "60",                         # 输入帧率
        "-f", "image2pipe",
        "-vcodec", "ppm",
        "-i", "-",                          # 从管道读取
        "-vcodec", "libx264",
        "-preset", "medium",
        "-pix_fmt", "yuv420p",
        "-crf", "18",                       # 质量 (越低越好)
        str(output_path)
    ]

    print("=" * 50)
    print("  Gource 视频生成器")
    print("=" * 50)
    print(f"项目目录: {PROJECT_DIR}")
    print(f"输出文件: {output_path}")
    print(f"分辨率: {resolution}")
    print()
    print("正在生成视频，请稍候...")
    print("(这可能需要几分钟)")
    print()

    # 管道连接 gource -> ffmpeg
    gource_proc = subprocess.Popen(
        gource_cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    ffmpeg_proc = subprocess.Popen(
        ffmpeg_cmd,
        stdin=gource_proc.stdout,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE
    )

    # 等待完成
    gource_proc.stdout.close()
    ffmpeg_output, ffmpeg_err = ffmpeg_proc.communicate()
    gource_proc.wait()

    if ffmpeg_proc.returncode == 0:
        print("✓ 视频生成成功!")
        print(f"  文件: {output_path}")
        print(f"  大小: {output_path.stat().st_size / 1024 / 1024:.1f} MB")
    else:
        print("✗ 视频生成失败")
        print(ffmpeg_err.decode())
        return False

    return True


def generate_preview_command():
    """生成预览命令（不输出视频，直接看）"""
    cmd = f'''gource "{PROJECT_DIR}" -1280x720 --seconds-per-day 0.3 --auto-skip-seconds 0.5 --hide filenames,dirnames --title "Cutie" --key'''
    print("\n预览命令 (不生成文件，直接看):")
    print(f"  {cmd}")


def main():
    print()

    # 解析参数
    output_path = DEFAULT_OUTPUT
    resolution = "1920x1080"

    if "--output" in sys.argv:
        idx = sys.argv.index("--output")
        if idx + 1 < len(sys.argv):
            output_path = Path(sys.argv[idx + 1])

    if "--resolution" in sys.argv:
        idx = sys.argv.index("--resolution")
        if idx + 1 < len(sys.argv):
            resolution = sys.argv[idx + 1]

    if "--preview" in sys.argv:
        generate_preview_command()
        return

    # 检查依赖
    if not check_dependencies():
        generate_preview_command()
        sys.exit(1)

    # 确保输出目录存在
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # 生成视频
    success = generate_video(output_path, resolution)

    if success:
        generate_preview_command()


if __name__ == "__main__":
    main()
