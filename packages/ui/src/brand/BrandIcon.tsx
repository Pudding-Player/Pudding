import avatar from "./avatar.png";

type IconProps = { className?: string };

/** 产品图标：桑多涅 Q 版头像。全尺寸源图在 assets/brand/avatar.png，此处为 256px 副本。 */
export function BrandIcon({ className }: IconProps) {
  return (
    <img
      src={avatar}
      alt=""
      draggable={false}
      className={className ? `app-brand-icon ${className}` : "app-brand-icon"}
    />
  );
}
