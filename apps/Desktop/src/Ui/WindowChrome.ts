import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type WindowAction = "close" | "minimize" | "toggleMaximize";

export function createWindowChrome(): HTMLDivElement {
  const chrome = document.createElement("div");
  chrome.className = "WindowChrome";

  const dragRegion = document.createElement("div");
  dragRegion.className = "WindowChromeDragRegion";
  dragRegion.setAttribute("data-tauri-drag-region", "");
  dragRegion.addEventListener("dblclick", () => {
    runWindowAction("toggleMaximize");
  });

  const controls = document.createElement("div");
  controls.className = "WindowChromeControls";
  controls.append(
    createWindowControl("Minimizar janela", "minimize", createMinimizeIcon()),
    createWindowControl(
      "Maximizar ou restaurar janela",
      "toggleMaximize",
      createMaximizeIcon()
    ),
    createWindowControl("Fechar janela", "close", createCloseIcon(), true)
  );

  chrome.append(dragRegion, controls);
  return chrome;
}

function createWindowControl(
  label: string,
  action: WindowAction,
  icon: SVGSVGElement,
  isClose = false
): HTMLButtonElement {
  const button = document.createElement("button");
  button.className = isClose ? "WindowChromeButton WindowChromeClose" : "WindowChromeButton";
  button.type = "button";
  button.setAttribute("aria-label", label);
  button.append(icon);
  button.addEventListener("click", () => {
    runWindowAction(action, button.parentElement);
  });
  return button;
}

function runWindowAction(action: WindowAction, controls: HTMLElement | null = null): void {
  if (!isTauri()) {
    return;
  }

  controls?.setAttribute("data-suppress-hover", "true");

  const appWindow = getCurrentWindow();
  const result = appWindow[action]();
  void result
    .catch((error: unknown) => {
      console.error(`Failed to execute window action '${action}'.`, error);
    })
    .finally(() => {
      window.requestAnimationFrame(() => {
        window.requestAnimationFrame(() => {
          controls?.removeAttribute("data-suppress-hover");
        });
      });
    });
}

function createWindowIcon(size: number): SVGSVGElement {
  const icon = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  icon.setAttribute("aria-hidden", "true");
  icon.setAttribute("fill", "none");
  icon.setAttribute("height", size.toString());
  icon.setAttribute("stroke", "currentColor");
  icon.setAttribute("stroke-linecap", "round");
  icon.setAttribute("stroke-linejoin", "round");
  icon.setAttribute("stroke-width", "1.7");
  icon.setAttribute("viewBox", "0 0 24 24");
  icon.setAttribute("width", size.toString());
  return icon;
}

function createMinimizeIcon(): SVGSVGElement {
  const icon = createWindowIcon(17);
  const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
  path.setAttribute("d", "M5 12h14");
  icon.append(path);
  return icon;
}

function createMaximizeIcon(): SVGSVGElement {
  const icon = createWindowIcon(13);
  const rectangle = document.createElementNS("http://www.w3.org/2000/svg", "rect");
  rectangle.setAttribute("height", "14");
  rectangle.setAttribute("rx", "1");
  rectangle.setAttribute("width", "14");
  rectangle.setAttribute("x", "5");
  rectangle.setAttribute("y", "5");
  icon.append(rectangle);
  return icon;
}

function createCloseIcon(): SVGSVGElement {
  const icon = createWindowIcon(18);

  for (const definition of ["M18 6 6 18", "m6 6 12 12"]) {
    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
    path.setAttribute("d", definition);
    icon.append(path);
  }

  return icon;
}
