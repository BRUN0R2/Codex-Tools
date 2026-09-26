import {
  canonicalizeLocale,
  resolveLocaleCatalog,
  resolveLocalePreference,
  type LocalePreference,
} from "./locale-selection";

type CatalogDirection = "ltr" | "rtl";

type Catalog = Readonly<{
  direction: CatalogDirection;
  locale: string;
  messages: Readonly<Record<string, string>>;
  name: string;
}>;

type EnglishCatalog = typeof import("./locales/en.json");
export type TranslationKey = keyof EnglishCatalog["messages"];

const DEFAULT_LOCALE = "en";
const LOCALE_STORAGE_KEY = "codex-tools.locale";
const MAXIMUM_LOCALE_LENGTH = 35;
const MAXIMUM_CATALOG_NAME_LENGTH = 80;
const catalogModules = import.meta.glob<unknown>("./locales/*.json", {
  eager: true,
  import: "default",
});
const catalogs = decodeCatalogs(catalogModules);
const englishCatalog = requiredEnglishCatalog();

let localeStorageIssue = false;
let localePreference = readLocalePreference();
export let activeCatalog = selectCatalog(localePreference);
let activeMessages = mergedMessages(activeCatalog);

export function availableCatalogs(): readonly Readonly<Pick<Catalog, "locale" | "name">>[] {
  return [...catalogs.values()].map(({ locale, name }) => ({ locale, name }));
}

export function getLocalePreference(): LocalePreference {
  return localePreference;
}

export function hasLocaleStorageIssue(): boolean {
  return localeStorageIssue;
}

export function setLocalePreference(preference: LocalePreference): void {
  if (preference !== "auto" && !catalogs.has(preference)) {
    throw new Error(`Translation locale ${preference} is unavailable.`);
  }
  localePreference = preference;
  activeCatalog = selectCatalog(preference);
  activeMessages = mergedMessages(activeCatalog);
  applyDocumentLocale();

  try {
    localStorage.setItem(LOCALE_STORAGE_KEY, preference);
    localeStorageIssue = false;
  } catch {
    localeStorageIssue = true;
  }
}

function readLocalePreference(): LocalePreference {
  try {
    return resolveLocalePreference(catalogs, localStorage.getItem(LOCALE_STORAGE_KEY));
  } catch {
    localeStorageIssue = true;
    return "auto";
  }
}

function selectCatalog(preference: LocalePreference): Catalog {
  return preference === "auto"
    ? resolveLocaleCatalog(catalogs, getRequestedLanguages(), DEFAULT_LOCALE)
    : catalogs.get(preference)!;
}

function mergedMessages(catalog: Catalog): Readonly<Record<string, string>> {
  return Object.freeze({ ...englishCatalog.messages, ...catalog.messages });
}

function requiredEnglishCatalog(): Catalog {
  const catalog = catalogs.get(DEFAULT_LOCALE);
  if (catalog === undefined) {
    throw new Error(`The required ${DEFAULT_LOCALE}.json translation catalog is missing.`);
  }
  return catalog;
}

export function translate(
  key: TranslationKey,
  replacements: Readonly<Record<string, string | number>> = {},
): string {
  const message = activeMessages[key];
  if (message === undefined) {
    throw new Error(`Translation key ${String(key)} is missing from the English catalog.`);
  }

  return formatMessage(message, replacements);
}

export function translatePlural(
  key: string,
  count: number,
  replacements: Readonly<Record<string, string | number>> = {},
): string {
  const category = new Intl.PluralRules(activeCatalog.locale).select(count);
  const message = activeMessages[`${key}.${category}`] ?? activeMessages[`${key}.other`];
  if (message === undefined) {
    throw new Error(`Plural translation ${key} has no ${category} or other message.`);
  }
  return formatMessage(message, replacements);
}

function formatMessage(
  message: string,
  replacements: Readonly<Record<string, string | number>>,
): string {
  return message.replace(/\{([A-Za-z][A-Za-z0-9_]*)\}/gu, (placeholder, name: string) => {
    const value = replacements[name];
    return value === undefined ? placeholder : String(value);
  });
}

export function applyDocumentLocale(): void {
  document.documentElement.lang = activeCatalog.locale;
  document.documentElement.dir = activeCatalog.direction;
}

function decodeCatalogs(
  modules: Readonly<Record<string, unknown>>,
): ReadonlyMap<string, Catalog> {
  const entries = Object.entries(modules).toSorted(([left], [right]) => left.localeCompare(right));
  const decoded = new Map<string, Catalog>();

  for (const [path, source] of entries) {
    const filename = /(?:^|[/\\])([^/\\]+)\.json$/u.exec(path)?.[1];
    if (filename === undefined || !isRecord(source)) {
      throw new Error(`Invalid translation catalog at ${path}.`);
    }
    if (
      typeof source["locale"] !== "string" ||
      source["locale"].length > MAXIMUM_LOCALE_LENGTH ||
      typeof source["name"] !== "string" ||
      source["name"].trim().length === 0 ||
      source["name"].length > MAXIMUM_CATALOG_NAME_LENGTH ||
      typeof source["direction"] !== "string" ||
      !isDirection(source["direction"]) ||
      !isRecord(source["messages"])
    ) {
      throw new Error(`Translation catalog ${path} has invalid metadata or messages.`);
    }
    const canonicalLocale = canonicalizeLocale(source["locale"]);
    if (canonicalLocale === null || canonicalLocale !== filename) {
      throw new Error(`The locale in ${path} must match its canonical filename.`);
    }
    if (decoded.has(canonicalLocale)) {
      throw new Error(`Translation locale ${canonicalLocale} is duplicated.`);
    }

    const messages: Record<string, string> = {};
    for (const [key, value] of Object.entries(source["messages"])) {
      if (typeof value !== "string" || value.length === 0) {
        throw new Error(`Translation ${canonicalLocale}.${key} must be a non-empty string.`);
      }
      messages[key] = value;
    }

    decoded.set(canonicalLocale, Object.freeze({
      direction: source["direction"],
      locale: canonicalLocale,
      messages: Object.freeze(messages),
      name: source["name"],
    }));
  }

  validateCatalogMessages(decoded);
  return decoded;
}

function validateCatalogMessages(catalogsToValidate: ReadonlyMap<string, Catalog>): void {
  const english = catalogsToValidate.get(DEFAULT_LOCALE);
  if (english === undefined) {
    throw new Error(`The required ${DEFAULT_LOCALE}.json translation catalog is missing.`);
  }

  for (const catalog of catalogsToValidate.values()) {
    for (const [key, message] of Object.entries(catalog.messages)) {
      const referenceMessage = english.messages[key];
      if (referenceMessage === undefined) {
        throw new Error(`Translation ${catalog.locale}.${key} is not defined in English.`);
      }
      const expectedPlaceholders = messagePlaceholders(referenceMessage);
      const receivedPlaceholders = messagePlaceholders(message);
      if (
        expectedPlaceholders.length !== receivedPlaceholders.length ||
        expectedPlaceholders.some((placeholder, index) => placeholder !== receivedPlaceholders[index])
      ) {
        throw new Error(`Translation ${catalog.locale}.${key} has different placeholders.`);
      }
    }
  }
}

function messagePlaceholders(message: string): readonly string[] {
  return [...message.matchAll(/\{([A-Za-z][A-Za-z0-9_]*)\}/gu)]
    .map((match) => match[1]!)
    .toSorted();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function getRequestedLanguages(): readonly string[] {
  if (typeof navigator === "undefined") return [];
  return [...navigator.languages, navigator.language];
}

function isDirection(value: string): value is CatalogDirection {
  return value === "ltr" || value === "rtl";
}
