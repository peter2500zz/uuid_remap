import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { Locale, TFunction, translate, LOCALES } from "./translations";

const STORAGE_KEY = "uuid_remap.locale";

function isLocale(value: string | null | undefined): value is Locale {
    return LOCALES.some(({ code }) => code === value);
}

function getInitialLocale(): Locale {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (isLocale(stored)) return stored;

    // 没有缓存过语言时按系统语言偏好自动判断，都匹配不上则回退英语
    for (const tag of navigator.languages ?? [navigator.language]) {
        const lang = tag?.toLowerCase().split("-")[0];
        if (isLocale(lang)) return lang;
    }
    return "en";
}

interface I18nContextType {
    locale: Locale;
    setLocale: (locale: Locale) => void;
    t: TFunction;
}

const I18nContext = createContext<I18nContextType>(null!);

function I18nProvider({ children }: { children: ReactNode }) {
    const [locale, setLocaleState] = useState<Locale>(getInitialLocale);

    useEffect(() => {
        document.documentElement.lang = locale;
    }, [locale]);

    const setLocale = (next: Locale) => {
        setLocaleState(next);
        localStorage.setItem(STORAGE_KEY, next);
    };

    const t: TFunction = (key, params) => translate(locale, key, params);

    return (
        <I18nContext.Provider value={{ locale, setLocale, t }}>
            {children}
        </I18nContext.Provider>
    );
}

function useI18n() {
    return useContext(I18nContext);
}

export { I18nProvider, useI18n };
