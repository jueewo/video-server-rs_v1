import Config from "../website.config.cjs";

type LocalizedString = string | Record<string, string>;

const defaultLang: string = Config.defaultLanguage?.locale ?? "en";

/**
 * Resolve a LocalizedString: plain strings pass through,
 * locale maps return the value for `lang` with fallback to default language.
 */
export function t(value: LocalizedString, lang: string): string {
    if (typeof value === "string") return value;
    return value[lang] ?? value[defaultLang] ?? Object.values(value)[0] ?? "";
}
