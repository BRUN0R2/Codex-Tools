import type { AppSection } from "../Domain/AppSection";
import {
  createProcessesIcon,
  createCleanupIcon,
  createUninstallIcon,
} from "./ApplicationIcons";

export type AppNavigationProps = Readonly<{
  activeSection: AppSection;
  onSelectSection: (section: AppSection) => void;
}>;

const NAVIGATION_ITEMS: readonly Readonly<{
  icon: () => SVGSVGElement;
  label: string;
  section: AppSection;
}>[] = [
  {
    icon: createProcessesIcon,
    label: "Processos",
    section: "Processes",
  },
  {
    icon: createCleanupIcon,
    label: "Limpeza",
    section: "Cleanup",
  },
  {
    icon: createUninstallIcon,
    label: "Desinstalacao",
    section: "Uninstall",
  },
];

export function createAppNavigation({
  activeSection,
  onSelectSection,
}: AppNavigationProps): HTMLElementTagNameMap["nav"] {
  const navigation = document.createElement("nav");
  navigation.className = "AppNavigation";
  navigation.setAttribute("aria-label", "Navegacao principal");

  for (const item of NAVIGATION_ITEMS) {
    const button = document.createElement("button");
    const isActive = item.section === activeSection;
    button.className =
      isActive ? "AppNavigationButton IsActive" : "AppNavigationButton";
    button.type = "button";
    const label = document.createElement("span");
    label.textContent = item.label;

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
