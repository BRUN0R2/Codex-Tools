import type { AppSection } from "../Domain/AppSection";
import { translate } from "../i18n/catalog";
import {
  createProcessesIcon,
  createCleanupIcon,
  createUninstallIcon,
  createSettingsIcon,
} from "./ApplicationIcons";

export type AppNavigationProps = Readonly<{
  activeSection: AppSection;
  onSelectSection: (section: AppSection) => void;
}>;

const NAVIGATION_ITEMS: readonly Readonly<{
  icon: () => SVGSVGElement;
  labelKey: "navigation.processes" | "navigation.cleanup" | "navigation.uninstall" | "navigation.settings";
  section: AppSection;
}>[] = [
  {
    icon: createProcessesIcon,
    labelKey: "navigation.processes",
    section: "Processes",
  },
  {
    icon: createCleanupIcon,
    labelKey: "navigation.cleanup",
    section: "Cleanup",
  },
  {
    icon: createUninstallIcon,
    labelKey: "navigation.uninstall",
    section: "Uninstall",
  },
  {
    icon: createSettingsIcon,
    labelKey: "navigation.settings",
    section: "Settings",
  },
];

export function createAppNavigation({
  activeSection,
  onSelectSection,
}: AppNavigationProps): HTMLElementTagNameMap["nav"] {
  const navigation = document.createElement("nav");
  navigation.className = "AppNavigation";
  navigation.setAttribute("aria-label", translate("app.navigationLabel"));

  for (const item of NAVIGATION_ITEMS) {
    const button = document.createElement("button");
    const isActive = item.section === activeSection;
    button.className =
      isActive ? "AppNavigationButton IsActive" : "AppNavigationButton";
    button.type = "button";
    const label = document.createElement("span");
    label.textContent = translate(item.labelKey);

    button.append(item.icon(), label);
    if (isActive) {
      button.setAttribute("aria-current", "page");
    }
    button.addEventListener("click", () => {
      onSelectSection(item.section);
    });
    navigation.append(button);
  }

  return navigation;
}
