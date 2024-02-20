import i18n from 'i18next';
import en from './en.json';
import ua from './ua.json';
import {initReactI18next} from "react-i18next";

i18n
    .use(initReactI18next)
    .init({
        interpolation: {
            escapeValue: false,
        },
        lng: navigator.language == "ru" ? "ua" : "en",
        supportedLngs: ["en", "ua"],
        // Using simple hardcoded resources for simple example
        resources: {
            en: {
                translation: en,
            },
            ua: {
                translation: ua,
            },
        },
    });

export default i18n