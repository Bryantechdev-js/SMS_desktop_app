import { useTranslation } from 'react-i18next';
import { Globe2 } from 'lucide-react';
import { SUPPORTED_LANGUAGES, type Language } from '../i18n/config';

export function LanguageSelector() {
  const { i18n } = useTranslation();
  const currentLanguage = (i18n.language as Language) || 'en';

  const handleLanguageChange = (lang: Language) => {
    i18n.changeLanguage(lang);
  };

  return (
    <div className="language-selector">
      <div className="language-dropdown">
        {Object.entries(SUPPORTED_LANGUAGES).map(([code, label]) => (
          <button
            key={code}
            onClick={() => handleLanguageChange(code as Language)}
            className={`language-option ${currentLanguage === code ? 'active' : ''}`}
            title={label}
          >
            <Globe2 size={16} />
            <span>{code.toUpperCase()}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
