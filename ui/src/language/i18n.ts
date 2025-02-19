import i18n from 'i18next'
import {initReactI18next} from "react-i18next";
import LanguageDetector from 'i18next-browser-languagedetector'
import de_translation from './json/de.json'
import en_translation from './json/en.json'
import fr_translation from './json/fr.json'
import pl_translation from './json/pl.json'

const resources = {
    de: {
        translation:de_translation
    },
    en:{
        translation: en_translation
    },
    fr:{
        translation: fr_translation
    },
    pl:{
        translation: pl_translation
    }
}

i18n
    .use(LanguageDetector)
    .use(initReactI18next)
    .init(
        {
            resources
        }
    )

export default i18n
