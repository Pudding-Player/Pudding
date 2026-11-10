"""从已批准的头像导出紧凑 Windows 图标，保留原始品牌素材。"""
from pathlib import Path

from PIL import Image, ImageOps

ROOT = Path(__file__).resolve().parent.parent


def main():
    source = Image.open(ROOT / "assets/brand/avatar.png").convert("RGBA")
    # alpha=1 的孤立噪点不属于图案边缘，不能据此保留整张源画布。
    bounds = source.getchannel("A").point(lambda value: 255 if value > 1 else 0).getbbox()
    if bounds is None:
        raise ValueError("品牌头像没有可见图案")
    artwork = source.crop(bounds)
    canvas = Image.new("RGBA", (256, 256))
    fitted = ImageOps.contain(artwork, (248, 248), Image.Resampling.LANCZOS)
    canvas.alpha_composite(fitted, ((256 - fitted.width) // 2, (256 - fitted.height) // 2))
    sizes = [(size, size) for size in (16, 20, 24, 32, 40, 48, 64, 128, 256)]
    for name in ("pudding.ico", "pudding-setup.ico"):
        canvas.save(ROOT / "assets" / name, format="ICO", sizes=sizes)
    print(f"Exported Windows icons from visible bounds {bounds}")


if __name__ == "__main__":
    main()
