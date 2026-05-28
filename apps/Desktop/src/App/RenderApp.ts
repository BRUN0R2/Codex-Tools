import { createShell } from "./Shell";

export function renderApp(root: HTMLElement | null): void {
  if (root === null) {
    throw new Error("Application root element was not found.");
  }

  root.replaceChildren(createShell());
}
