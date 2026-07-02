import en from "./lang/en";
import zh from "./lang/zh";
import ja from "./lang/ja";

type Locale = "en" | "zh" | "ja";

const LOCALES: { code: Locale; label: string }[] = [
    { code: "en", label: "English" },
    { code: "zh", label: "中文" },
    { code: "ja", label: "日本語" },
];

const translations: Record<Locale, Record<string, string>> = { en, zh, ja };

type TFunction = (key: string, params?: Record<string, string | number>) => string;

function translate(locale: Locale, key: string, params?: Record<string, string | number>): string {
    let text = translations[locale][key] ?? translations.en[key] ?? key;
    if (params) {
        for (const [name, value] of Object.entries(params)) {
            text = text.split(`{${name}}`).join(String(value));
        }
    }
    return text;
}

export { translations, translate, LOCALES };
export type { Locale, TFunction };
