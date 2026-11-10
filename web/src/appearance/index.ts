import { preference } from "./preference";
import { motionDisabled } from "@pudding/ui/motion";

export type ThemeId = "light" | "dark";
export type ThemeOrigin = { x: number; y: number };
const theme = preference<ThemeId>(
  "pudding-theme",
  (stored) => (stored === "dark" ? "dark" : "light"),
  (value) => {
    document.documentElement.dataset.theme = value;
    window.ipc?.postMessage(
      JSON.stringify({
        id: crypto.randomUUID(),
        method: "window.setTheme",
        params: { theme: value },
      }),
    );
  },
);
export const readTheme = theme.read;
export const subscribeTheme = theme.subscribe;
export const restoreTheme = theme.restore;
export function useTheme(): ThemeId {
  return theme.use();
}

type Scope = HTMLElement & {
  startViewTransition?: (update: () => void) => ViewTransition;
};

type Wave = {
  transition: ViewTransition;
  animation?: Animation;
  from: ThemeId;
  to: ThemeId;
  point: ThemeOrigin;
  layer: "old" | "new";
};
let wave: Wave | undefined;
let generation = 0;
const promoted = new Map<HTMLElement, () => void>();

function promoteControls() {
  for (const button of document.querySelectorAll<HTMLElement>(
    ".app-theme-switch",
  )) {
    if (
      !button.getClientRects().length ||
      button.closest("[inert],[aria-hidden=true]")
    )
      continue;
    const node =
      button.closest<HTMLElement>("[data-theme-control-group]") ?? button;
    if (promoted.has(node) || node.matches(":popover-open")) continue;
    const box = node.getBoundingClientRect(),
      style = getComputedStyle(node);
    const matrix = new DOMMatrixReadOnly(style.transform);
    const padding = style.padding,
      borderWidth = style.borderWidth,
      borderStyle = style.borderStyle;
    const borderBox = style.boxSizing === "border-box";
    const width =
        parseFloat(style.width) +
        (borderBox
          ? 0
          : parseFloat(style.paddingLeft) +
            parseFloat(style.paddingRight) +
            parseFloat(style.borderLeftWidth) +
            parseFloat(style.borderRightWidth)),
      height =
        parseFloat(style.height) +
        (borderBox
          ? 0
          : parseFloat(style.paddingTop) +
            parseFloat(style.paddingBottom) +
            parseFloat(style.borderTopWidth) +
            parseFloat(style.borderBottomWidth));
    const [originX = 0, originY = 0] = style.transformOrigin
      .split(" ")
      .map(parseFloat);
    const properties = [
      "position",
      "left",
      "top",
      "right",
      "bottom",
      "width",
      "height",
      "margin",
      "box-sizing",
      "padding",
      "border-width",
      "border-style",
    ];
    const original = properties.map((name) => [
      name,
      node.style.getPropertyValue(name),
      node.style.getPropertyPriority(name),
    ]);
    const attribute = node.getAttribute("popover");
    const placeholder = document.createElement("div");
    placeholder.setAttribute("aria-hidden", "true");
    if (!["absolute", "fixed"].includes(style.position)) {
      Object.assign(placeholder.style, {
        boxSizing: "border-box",
        width: `${width}px`,
        height: `${height}px`,
        margin: style.margin,
        flex: style.flex,
        gridColumn: style.gridColumn,
        gridRow: style.gridRow,
        visibility: "hidden",
        pointerEvents: "none",
      });
      node.before(placeholder);
    }
    node.setAttribute("popover", "manual");
    Object.assign(node.style, {
      position: "fixed",
      left: `${box.left - originX * (1 - matrix.a) - matrix.e}px`,
      top: `${box.top - originY * (1 - matrix.d) - matrix.f}px`,
      right: "auto",
      bottom: "auto",
      width: `${width}px`,
      height: `${height}px`,
      margin: "0",
      boxSizing: "border-box",
      // 顶层控件保持原边框盒，阻止 popover 的默认边框和内边距改变内容尺寸。
      padding,
      borderWidth,
      borderStyle,
    });
    node.showPopover();
    promoted.set(node, () => {
      const focused = node.contains(document.activeElement)
        ? (document.activeElement as HTMLElement)
        : null;
      if (node.matches(":popover-open")) node.hidePopover();
      if (attribute === null) node.removeAttribute("popover");
      else node.setAttribute("popover", attribute);
      for (const [name, value, priority] of original) {
        if (value) node.style.setProperty(name!, value, priority);
        else node.style.removeProperty(name!);
      }
      placeholder.remove();
      if (focused?.isConnected) focused.focus({ preventScroll: true });
    });
  }
}
function releaseControls() {
  for (const restore of promoted.values()) restore();
  promoted.clear();
  delete document.body.dataset.themeReveal;
  delete document.body.dataset.themeRevealLayer;
}

export function writeTheme(next: ThemeId, origin?: ThemeOrigin): Promise<void> {
  // 捕获范围限定在 body，不回退到 document/html，避免根层截获控件连点。
  const scope = document.body as Scope;
  if (wave?.to === next && readTheme() === next)
    return wave.transition.finished.catch(() => undefined);
  const token = ++generation;
  let point = origin ?? { x: innerWidth / 2, y: innerHeight / 2 };
  let carry: { radius: number; inside: ThemeId } | undefined;
  const previous = wave;
  if (previous?.animation) {
    const clip = getComputedStyle(
      scope,
      `::view-transition-${previous.layer}(root)`,
    ).clipPath;
    const match = /^circle\(([\d.]+)px/.exec(clip);
    if (match) {
      point = previous.point;
      carry = {
        radius: Number(match[1]),
        inside: previous.layer === "new" ? previous.to : previous.from,
      };
    }
  }
  previous?.transition.skipTransition();
  previous?.animation?.cancel();
  wave = undefined;
  if (
    !scope.startViewTransition ||
    motionDisabled() ||
    document.documentElement.dataset.theme === next
  ) {
    theme.write(next);
    return (
      previous?.transition.finished.catch(() => undefined) ?? Promise.resolve()
    ).then(() => {
      if (generation === token) releaseControls();
    });
  }
  promoteControls();
  const bounds = scope.getBoundingClientRect();
  const x = point.x - bounds.left,
    y = point.y - bounds.top;
  const radius = Math.hypot(
    Math.max(x, bounds.width - x),
    Math.max(y, bounds.height - y),
  );
  const layer = carry && carry.inside !== next ? "old" : "new";
  const from = readTheme();
  scope.dataset.themeReveal = "true";
  scope.dataset.themeRevealLayer = layer;
  let transition: ViewTransition;
  try {
    transition = scope.startViewTransition(() => {
      if (generation === token) theme.write(next);
    });
  } catch {
    theme.write(next);
    releaseControls();
    return Promise.resolve();
  }
  const current: Wave = { transition, from, to: next, point, layer };
  wave = current;
  void transition.ready
    .then(() => {
      if (generation !== token) return;
      const beginning = Math.min(carry?.radius ?? 0, radius),
        end = layer === "new" ? radius : 0;
      current.animation = scope.animate(
        {
          clipPath: [
            `circle(${beginning}px at ${x}px ${y}px)`,
            `circle(${end}px at ${x}px ${y}px)`,
          ],
        },
        {
          duration: Math.max(100, (560 * Math.abs(end - beginning)) / radius),
          easing: "cubic-bezier(0.22,0.08,0.16,1)",
          pseudoElement: `::view-transition-${layer}(root)`,
        },
      );
      for (const animation of scope.getAnimations({ subtree: true })) {
        if (
          animation instanceof CSSAnimation &&
          animation.animationName === "theme-scope-hold"
        )
          animation.cancel();
      }
    })
    .catch(() => transition.skipTransition());
  return transition.finished
    .catch(() => undefined)
    .then(() => {
      if (generation === token) {
        wave = undefined;
        releaseControls();
      }
    });
}
