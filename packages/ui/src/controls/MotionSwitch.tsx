import { Fragment, useLayoutEffect, useRef, type ReactNode } from "react";
import { animateVisual, cancelVisual } from "../motion";

export type MotionKind = "page" | "panel";

/** 容器身份稳定；语义视图切换时仅更新子树，动画从当前呈现状态接续。 */
export function MotionSwitch({
  viewKey,
  kind = "page",
  children,
  className,
}: {
  viewKey: string;
  kind?: MotionKind;
  children: ReactNode;
  className?: string;
}) {
  const root = useRef<HTMLDivElement>(null);
  useLayoutEffect(() => {
    const node = root.current;
    if (!node) return;
    if (!node.getAnimations().length) {
      node.style.opacity = "0";
      node.style.transform = `translateY(${kind === "page" ? 8 : 6}px)`;
    }
    void animateVisual(node, { opacity: 1 }, kind === "page" ? 320 : 240).catch(
      () => {},
    );
  }, [viewKey, kind]);
  useLayoutEffect(() => {
    const node = root.current;
    return () => cancelVisual(node);
  }, []);
  return (
    <div
      ref={root}
      className={["motion-switch", className].filter(Boolean).join(" ")}
      data-motion={kind}
    >
      <Fragment key={viewKey}>{children}</Fragment>
    </div>
  );
}
