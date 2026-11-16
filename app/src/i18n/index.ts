// vue-i18n setup: every user-facing string lives in the message files of this folder.
// en.json is the reference schema and the fallback; it.json mirrors its keys.
// The language follows the OS / browser locale (WebView2 reports the system one).
import { createI18n } from "vue-i18n";
import en from "./en.json";
import it from "./it.json";

export type MessageSchema = typeof en;

export const LOCALES = ["en", "it"] as const;
export type Locale = (typeof LOCALES)[number];

/** First supported language among the user's preferred ones, else English. */
function detectLocale(): Locale {
  const preferred = navigator.languages?.length ? navigator.languages : [navigator.language];
  for (const tag of preferred) {
    const base = tag?.toLowerCase().split("-")[0];
    const match = LOCALES.find((l) => l === base);
    if (match) return match;
  }
  return "en";
}

export const LOCALE: Locale = detectLocale();
document.documentElement.lang = LOCALE;

export const i18n = createI18n<[MessageSchema], Locale, false>({
  legacy: false,
  locale: LOCALE,
  fallbackLocale: "en",
  messages: { en, it },
});

/** Translation outside components (bridge helpers, formatters). */
export const t = i18n.global.t;
