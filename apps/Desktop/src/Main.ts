import { renderApp } from "./App/RenderApp";
import { applyDocumentLocale } from "./i18n/catalog";

applyDocumentLocale();
renderApp(document.querySelector<HTMLElement>("#app"));
