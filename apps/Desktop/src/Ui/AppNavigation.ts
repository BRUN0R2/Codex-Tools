import type { AppSection } from "../Domain/AppSection";

export type AppNavigationProps = Readonly<{
  activeSection: AppSection;
  onSelectSection: (section: AppSection) => void;
}>;

const NAVIGATION_ITEMS: readonly Readonly<{
  description: string;
  label: string;
  section: AppSection;
}>[] = [
  {
    description: "Execucao e processos",
    label: "Admin",
    section: "Admin",
  },
  {
    description: "Dados locais",
    label: "Limpeza",
    section: "Cleanup",
  },
];

export function createAppNavigation({
  activeSection,
  onSelectSection,
}: AppNavigationProps): HTMLElementTagNameMap["nav"] {
  const navigation = document.createElement("nav");
  navigation.className = "AppNavigation";
  navigation.setAttribute("aria-label", "Secoes");
  navigation.setAttribute("role", "tablist");

  for (const item of NAVIGATION_ITEMS) {
    const button = document.createElement("button");
    const isActive = item.section === activeSection;
    button.className =
      isActive ? "AppNavigationButton IsActive" : "AppNavigationButton";
    button.type = "button";
    const label = document.createElement("strong");
    label.textContent = item.label;

    const description = document.createElement("span");
    description.textContent = item.description;

    button.append(label, description);
    button.setAttribute("aria-selected", isActive ? "true" : "false");
    button.setAttribute("role", "tab");
    button.addEventListener("click", () => {
      onSelectSection(item.section);
    });
    navigation.append(button);
  }

  return navigation;
}
