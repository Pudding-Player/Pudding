import { useLayoutEffect, useRef, type RefObject } from "react";
import { motionDisabled } from "@pudding/ui/motion";

/** 只在目标改变时重排一次，其余位移由合成层接续当前呈现位置。 */
export function useSidebarMotion({
  shell,
  sidebar,
  main,
  docked,
  visible,
  width,
  resizing,
}: {
  shell: RefObject<HTMLDivElement | null>;
  sidebar: RefObject<HTMLElement | null>;
  main: RefObject<HTMLElement | null>;
  docked: boolean;
  visible: boolean;
  width: number;
  resizing: boolean;
}) {
  const previous = useRef<{
    offset: number;
    sidebarX: number;
    resizing: boolean;
  } | null>(null);
  const motions = useRef<{ sidebar?: Animation; main?: Animation }>({});

  useLayoutEffect(() => {
    const panel = sidebar.current;
    const content = main.current;
    if (!panel || !content || !shell.current) return;
    const offset = docked ? width : 0;
    const sidebarX = visible ? 0 : -panel.getBoundingClientRect().width;
    const last = previous.current;
    previous.current = { offset, sidebarX, resizing };
    if (!last) return;

    // 先读两个正在播放的变换，再取消旧动画，避免反向时退回端点。
    const position = (node: HTMLElement) =>
      new DOMMatrixReadOnly(getComputedStyle(node).transform).m41;
    const fromSidebar = motions.current.sidebar
      ? position(panel)
      : last.sidebarX;
    const fromMain =
      last.offset - offset + (motions.current.main ? position(content) : 0);
    const style = getComputedStyle(shell.current);
    const time = style.getPropertyValue("--motion-slow").trim();
    const duration = parseFloat(time) * (time.endsWith("ms") ? 1 : 1000);
    const easing =
      style.getPropertyValue("--ease-enter").trim() ||
      "cubic-bezier(0.22,0.08,0.16,1)";
    motions.current.sidebar?.cancel();
    motions.current.main?.cancel();
    motions.current = {};
    if (resizing || last.resizing || motionDisabled()) return;

    const play = (
      node: HTMLElement,
      key: "sidebar" | "main",
      from: number,
      to: number,
    ) => {
      if (Math.abs(from - to) < 0.25) return;
      const animation = node.animate(
        [
          { transform: `translate3d(${from}px, 0, 0)` },
          { transform: `translate3d(${to}px, 0, 0)` },
        ],
        { duration: Number.isFinite(duration) ? duration : 320, easing },
      );
      motions.current[key] = animation;
      void animation.finished.then(
        () => {
          if (motions.current[key] === animation) delete motions.current[key];
        },
        () => {},
      );
    };
    play(panel, "sidebar", fromSidebar, sidebarX);
    play(content, "main", fromMain, 0);
  }, [shell, sidebar, main, docked, visible, width, resizing]);

  useLayoutEffect(
    () => () => {
      motions.current.sidebar?.cancel();
      motions.current.main?.cancel();
      motions.current = {};
    },
    [],
  );
}
