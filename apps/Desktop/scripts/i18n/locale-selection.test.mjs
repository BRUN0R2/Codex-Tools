import assert from "node:assert/strict";
import test from "node:test";
import { resolveLocaleCatalog } from "../../src/i18n/locale-selection.ts";

const catalogs = new Map([
  ["en", { locale: "en", name: "English" }],
  ["pt-BR", { locale: "pt-BR", name: "Português (Brasil)" }],
]);

test("selects an exact system locale", () => {
  assert.equal(resolveLocaleCatalog(catalogs, ["pt-BR"], "en").locale, "pt-BR");
});

test("selects the available catalog for a matching language family", () => {
  assert.equal(resolveLocaleCatalog(catalogs, ["pt-PT"], "en").locale, "pt-BR");
});

test("uses English when no supported language matches", () => {
  assert.equal(resolveLocaleCatalog(catalogs, ["fr-CA"], "en").locale, "en");
});

test("ignores invalid language identifiers and falls back to English", () => {
  assert.equal(resolveLocaleCatalog(catalogs, ["not_a_locale"], "en").locale, "en");
});
