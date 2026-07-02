import { createContext, useContext, useEffect, useState, ReactNode } from "react";
import { Locale, TFunction, translate } from "./translations";

const STORAGE_KEY = "uuid_remap.locale";

function getInitialLocale(): Locale {
    const stored = localStorage.getItem(STORAGE_KEY);
    return stored === "en" || stored === "zh" ? stored : "en";
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
