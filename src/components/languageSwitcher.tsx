import { useEffect, useRef, useState } from "react";
import { useI18n } from "../i18n/context";
import { LOCALES } from "../i18n/translations";

// 不使用 daisyUI 的 dropdown/dropdown-content：它靠 :focus-within 控制显隐，
// 而 WKWebView 中原生 <button> 点击后不会获得焦点，菜单会被 CSS 强制 display:none。
// 改为完全由 React 状态驱动 + 手动绝对定位
function LanguageSwitcher() {
    const { locale, setLocale, t } = useI18n();
    const [isOpen, setIsOpen] = useState(false);
    const rootRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        if (!isOpen) return;

        const onPointerDown = (event: PointerEvent) => {
            if (rootRef.current && !rootRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };
        document.addEventListener("pointerdown", onPointerDown);
        return () => document.removeEventListener("pointerdown", onPointerDown);
    }, [isOpen]);

    return (
        <div ref={rootRef} className="absolute top-4 left-4 z-30">
            <button
                type="button"
                className="btn btn-square btn-ghost border border-base-300"
                aria-label={t("language.switcherAria")}
                aria-expanded={isOpen}
                onClick={() => setIsOpen(prev => !prev)}
            >
                <span className="relative inline-block w-7 h-7 leading-none select-none">
                    <span className="absolute top-0 left-0 text-lg font-bold">文</span>
                    <span className="absolute bottom-0 right-0 text-xs font-semibold">A</span>
                </span>
            </button>

            {isOpen && (
                <ul className="menu absolute top-full left-0 mt-1 bg-base-100 rounded-box w-32 p-2 shadow-sm border border-base-300">
                    {LOCALES.map(({ code, label }) => (
                        <li key={code}>
                            <button
                                type="button"
                                className={locale === code ? "active" : ""}
                                onClick={() => {
                                    setLocale(code);
                                    setIsOpen(false);
                                }}
                            >
                                {label}
                            </button>
                        </li>
                    ))}
                </ul>
            )}
        </div>
    );
}

export default LanguageSwitcher;
