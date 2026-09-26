export type LocaleCatalog = Readonly<{ locale: string }>;

export function resolveLocaleCatalog<Catalog extends LocaleCatalog>(
  catalogs: ReadonlyMap<string, Catalog>,
  requestedLanguages: readonly string[],
  defaultLocale: string,
): Catalog {
  for (const requestedLanguage of requestedLanguages) {
    const canonicalLocale = canonicalizeLocale(requestedLanguage);
    if (canonicalLocale === null) continue;
    const exactMatch = catalogs.get(canonicalLocale);
    if (exactMatch !== undefined) return exactMatch;
  }

  for (const requestedLanguage of requestedLanguages) {
    const canonicalLocale = canonicalizeLocale(requestedLanguage);
    if (canonicalLocale === null) continue;
    const language = new Intl.Locale(canonicalLocale).language;
    const languageMatch = [...catalogs.values()].find(
      (catalog) => new Intl.Locale(catalog.locale).language === language,
    );
    if (languageMatch !== undefined) return languageMatch;
  }

  const fallback = catalogs.get(defaultLocale);
  if (fallback === undefined) {
    throw new Error(`The required ${defaultLocale}.json translation catalog is missing.`);
  }
  return fallback;
}

export function canonicalizeLocale(locale: string): string | null {
  try {
    return Intl.getCanonicalLocales(locale)[0] ?? null;
  } catch {
    return null;
  }
}
