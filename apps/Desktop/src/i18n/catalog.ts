import { canonicalizeLocale, resolveLocaleCatalog } from "./locale-selection";

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
const MAXIMUM_LOCALE_LENGTH = 35;
const MAXIMUM_CATALOG_NAME_LENGTH = 80;
const catalogModules = import.meta.glob<unknown>("./locales/*.json", {
  eager: true,
  import: "default",
});
const catalogs = decodeCatalogs(catalogModules);
const englishCatalog = catalogs.get(DEFAULT_LOCALE);

if (englishCatalog === undefined) {
  throw new Error(`The required ${DEFAULT_LOCALE}.json translation catalog is missing.`);
}

export const activeCatalog = resolveLocaleCatalog(
  catalogs,
  getRequestedLanguages(),
  DEFAULT_LOCALE,
);
const activeMessages = Object.freeze({
  ...englishCatalog.messages,
  ...activeCatalog.messages,
});

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
