import { mountApp } from "./AppController";

export function renderApp(root: HTMLElement | null): void {
  if (root === null) {
    throw new Error("Application root element was not found.");
  }

  mountApp(root);
}
