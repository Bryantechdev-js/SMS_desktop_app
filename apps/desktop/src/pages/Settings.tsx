import { useTranslation } from 'react-i18next';
import { useState } from 'react';
import { LanguageSelector } from '../components/LanguageSelector';

export function SettingsPage() {
  const { t } = useTranslation();
  const [businessName, setBusinessName] = useState('Complexe SMS');
  const [address, setAddress] = useState('');
  const [phone, setPhone] = useState('');
  const [email, setEmail] = useState('');
  const [successMessage, setSuccessMessage] = useState('');

  const handleSaveSettings = () => {
    setSuccessMessage(t('success.settingsSaved'));
    setTimeout(() => setSuccessMessage(''), 3000);
  };

  return (
    <div className="settings-page">
      <div className="settings-grid">
        {/* Business Information */}
        <div className="panel">
          <div className="panel-head">
            <div>
              <h2>{t('settings.business')}</h2>
              <p>Configure your bakery information</p>
            </div>
          </div>
          <form className="settings-form">
            <label>
              {t('settings.businessName')}
              <input
                type="text"
                value={businessName}
                onChange={(e) => setBusinessName(e.target.value)}
                placeholder="Your bakery name"
              />
            </label>
            <label>
              {t('settings.address')}
              <input
                type="text"
                value={address}
                onChange={(e) => setAddress(e.target.value)}
                placeholder="Business address"
              />
            </label>
            <label>
              {t('settings.phone')}
              <input
                type="tel"
                value={phone}
                onChange={(e) => setPhone(e.target.value)}
                placeholder="Contact phone"
              />
            </label>
            <label>
              {t('settings.email')}
              <input
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="Contact email"
              />
            </label>
            <button
              type="button"
              className="btn btn-primary"
              onClick={handleSaveSettings}
            >
              {t('common.save')}
            </button>
          </form>
        </div>

        {/* Preferences */}
        <div className="panel">
          <div className="panel-head">
            <div>
              <h2>{t('settings.preferences')}</h2>
              <p>Personalize your workspace</p>
            </div>
          </div>
          <form className="settings-form">
            <label>
              {t('settings.language')}
              <LanguageSelector />
            </label>
            <label>
              {t('settings.currency')}
              <select disabled>
                <option>XAF - Central African franc</option>
              </select>
              <small>Currency is fixed to XAF for this region</small>
            </label>
          </form>
        </div>

        {/* Security */}
        <div className="panel">
          <div className="panel-head">
            <div>
              <h2>{t('settings.security')}</h2>
              <p>Manage your account security</p>
            </div>
          </div>
          <form className="settings-form">
            <label>
              {t('settings.currentPassword')}
              <input
                type="password"
                placeholder="Enter current password"
                disabled
              />
              <small>Password changes coming in next update</small>
            </label>
            <label>
              {t('settings.newPassword')}
              <input
                type="password"
                placeholder="Enter new password"
                disabled
              />
            </label>
          </form>
        </div>

        {/* About */}
        <div className="panel">
          <div className="panel-head">
            <div>
              <h2>{t('settings.aboutApp')}</h2>
              <p>Application information</p>
            </div>
          </div>
          <div className="about-info">
            <p>
              <strong>Complexe SMS</strong>
            </p>
            <p>{t('settings.copyright')}</p>
            <p>{t('settings.version')}: 0.1.0</p>
            <p style={{ marginTop: '1rem', fontSize: '0.85rem', color: '#666' }}>
              Professional bakery operations and inventory management system.
              Built with React, Tauri, and Rust.
            </p>
          </div>
        </div>
      </div>

      {successMessage && (
        <div className="notice success-notice" style={{ position: 'fixed', top: '80px', right: '20px' }}>
          {successMessage}
        </div>
      )}
    </div>
  );
}
