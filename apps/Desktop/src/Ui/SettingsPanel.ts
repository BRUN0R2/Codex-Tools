import {
  activeCatalog,
  availableCatalogs,
  getLocalePreference,
  hasLocaleStorageIssue,
  translate,
} from "../i18n/catalog";
import type { LocalePreference } from "../i18n/locale-selection";

export function createSettingsPanel(
  onSelectLocale: (preference: LocalePreference) => void,
): HTMLElementTagNameMap["section"] {
  const panel = document.createElement("section");
  panel.className = "Panel SettingsPanel";

  const heading = document.createElement("div");
  heading.className = "SectionHeading";
  const title = document.createElement("h2");
  title.className = "SectionTitle";
  title.textContent = translate("settings.title");
  const description = document.createElement("p");
  description.className = "SectionDescription";
  description.textContent = translate("settings.description");
  heading.append(title, description);

  const field = document.createElement("label");
  field.className = "SettingsField";
  const label = document.createElement("span");
  label.textContent = translate("settings.language");
  const select = document.createElement("select");
  select.className = "FieldSelect";
  const automatic = document.createElement("option");
  automatic.value = "auto";
  automatic.textContent = translate("settings.autoLanguage");
  select.append(automatic);
  for (const catalog of availableCatalogs()) {
    const option = document.createElement("option");
    option.value = catalog.locale;
    option.textContent = catalog.name;
    select.append(option);
  }
  select.value = getLocalePreference();
  select.addEventListener("change", () => onSelectLocale(select.value));
  field.append(label, select);

  const active = document.createElement("p");
  active.className = "SettingsDetail";
  active.textContent = translate("settings.activeLanguage", { name: activeCatalog.name });

  panel.append(heading, field, active);
  if (hasLocaleStorageIssue()) {
    const warning = document.createElement("p");
    warning.className = "SettingsWarning";
    warning.textContent = translate("settings.storageIssue");
    panel.append(warning);
  }
  return panel;
}
