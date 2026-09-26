const SVG_NAMESPACE = "http://www.w3.org/2000/svg";

function createApplicationIcon(pathDefinitions: readonly string[]): SVGSVGElement {
  const icon = document.createElementNS(SVG_NAMESPACE, "svg");
  icon.classList.add("ApplicationIcon");
  icon.setAttribute("aria-hidden", "true");
  icon.setAttribute("fill", "none");
  icon.setAttribute("stroke", "currentColor");
  icon.setAttribute("stroke-linecap", "round");
  icon.setAttribute("stroke-linejoin", "round");
  icon.setAttribute("stroke-width", "1.8");
  icon.setAttribute("viewBox", "0 0 24 24");

  for (const definition of pathDefinitions) {
    const path = document.createElementNS(SVG_NAMESPACE, "path");
    path.setAttribute("d", definition);
    icon.append(path);
  }

  return icon;
}

export function createBrandIcon(): SVGSVGElement {
  return createApplicationIcon([
    "m8 7-4 5 4 5",
    "m16 7 4 5-4 5",
    "M10.5 19h3",
  ]);
}

export function createProcessesIcon(): SVGSVGElement {
  return createApplicationIcon([
    "M4.5 5h15v14h-15V5Z",
    "m7 9 3 3-3 3",
    "M12.5 15h4.5",
  ]);
}

export function createCleanupIcon(): SVGSVGElement {
  return createApplicationIcon([
    "m14.5 4.5 5 5-8.5 8.5H6v-5l8.5-8.5Z",
    "m12 7 5 5",
    "M4 20h11",
  ]);
}

export function createUninstallIcon(): SVGSVGElement {
  return createApplicationIcon([
    "M4 7h16",
    "M9 7V4h6v3",
    "m7 7 1 13h8l1-13",
    "M10 11v5",
    "M14 11v5",
  ]);
}

export function createRefreshIcon(): SVGSVGElement {
  return createApplicationIcon([
    "M20 7v5h-5",
    "M19 12a7 7 0 1 0-2 5",
  ]);
}

export function createLaunchIcon(): SVGSVGElement {
  return createApplicationIcon([
    "M5 12h14",
    "m14 7 5 5-5 5",
  ]);
}
