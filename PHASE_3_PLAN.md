# Phase 3: English/French Internationalization (i18n)
## Implementation Plan

**Objective**: Deliver complete bilingual support (English/French) across frontend, backend, and user interface.

**Scope**:
- Centralized translation dictionary system
- Language selection UI (settings menu)
- Language persistence (localStorage)
- Number, date, and currency formatting
- Full UI translation (navigation, labels, buttons, messages)
- API response message translation
- Receipt translation
- Error message translation
- Validation message translation

---

## Key Requirements

### 1. Translation Coverage
- ✅ Navigation labels
- ✅ Page titles
- ✅ Form labels
- ✅ Button text
- ✅ Placeholder text
- ✅ Validation messages
- ✅ Error messages
- ✅ Success messages
- ✅ Table headers
- ✅ Empty states
- ✅ Receipt text
- ✅ Settings labels
- ✅ Help text

### 2. Formatting
- ✅ Numbers (1,234.56 vs 1 234,56)
- ✅ Dates (MM/DD/YYYY vs DD/MM/YYYY)
- ✅ Currency (XAF formatting same, but context text different)
- ✅ Time (24-hour format standard in both)

### 3. Technical Implementation
- ✅ Lightweight i18n library (avoid heavy dependencies)
- ✅ Server-side translation (optional, for future)
- ✅ Client-side translation (frontend)
- ✅ Language preference persistence
- ✅ No page reload on language switch
- ✅ RTL support consideration (not required for EN/FR)

---

## Translation Libraries Evaluation

### Option 1: i18next (Recommended)
**Pros**:
- Industry standard
- Excellent React support
- Namespace support
- Lazy loading
- CLI tools

**Cons**:
- Larger bundle size
- More complex setup

**Decision**: Use i18next for robustness

### Option 2: react-intl
**Pros**:
- Deep React integration
- Excellent date/number formatting
- ICU format support

**Cons**:
- Larger bundle
- More overhead

**Decision**: i18next preferred, but react-intl worth evaluating

### Option 3: Custom Lightweight Solution
**Pros**:
- Small bundle
- Full control
- Simple setup

**Cons**:
- No ecosystem
- Missing formatting utilities
- Maintenance burden

**Decision**: Use i18next, can add lightweight fallback if needed

---

## Implementation Strategy

### Phase 3a: Frontend (React)
1. Install i18next + React integration
2. Create translation JSON files (en.json, fr.json)
3. Initialize i18next in App.tsx
4. Add language selector component
5. Replace all hardcoded strings with translation keys
6. Implement date/number/currency formatting
7. Test language switching

### Phase 3b: Translation Content
1. Extract all translatable strings from Phase 1 & 2
2. Create English base dictionary
3. Translate to French
4. Create formatting rules (dates, numbers, currency)
5. Review translations with native speakers (if available)

### Phase 3c: Backend Support (Optional)
1. Add language header to API responses
2. Translate error messages on server
3. API responses use translation keys
4. Frontend resolves translation keys

### Phase 3d: Settings & Persistence
1. Add language selector to Settings page
2. Store preference in localStorage
3. Load preference on app startup
4. Restore session language on reload

---

## Translation Dictionary Structure

```json
{
  "common": {
    "save": "Save",
    "cancel": "Cancel",
    "delete": "Delete",
    "edit": "Edit",
    "add": "Add",
    "search": "Search",
    "error": "Error",
    "success": "Success",
    "loading": "Loading...",
    "empty": "No records found"
  },
  
  "navigation": {
    "overview": "Overview",
    "inventory": "Inventory",
    "pos": "Point of Sale",
    "production": "Production",
    "procurement": "Procurement",
    "cash": "Cash",
    "settings": "Settings"
  },
  
  "inventory": {
    "products": "Products",
    "balances": "Stock Balances",
    "movements": "Movements",
    "lowStock": "Low Stock",
    "addProduct": "Add Product",
    "productName": "Product Name",
    "sku": "SKU",
    "quantity": "Quantity",
    "onHand": "On-Hand",
    "status": "Status",
    "inStock": "In Stock",
    "lowStockAlert": "Low Stock Alert"
  },
  
  "pos": {
    "pointOfSale": "Point of Sale",
    "searchProducts": "Search Products",
    "cart": "Cart",
    "subtotal": "Subtotal",
    "discount": "Discount",
    "tax": "Tax",
    "total": "Total",
    "paymentMethod": "Payment Method",
    "completeSale": "Complete Sale",
    "receipt": "Receipt"
  },
  
  "validation": {
    "required": "This field is required",
    "invalidSku": "Invalid SKU",
    "negativeQuantity": "Quantity must be positive",
    "insufficientStock": "Insufficient stock"
  },
  
  "errors": {
    "apiUnavailable": "API unavailable. Please try again later.",
    "invalidCredentials": "Invalid username or password",
    "unauthorized": "You do not have permission to perform this action"
  }
}
```

---

## API Endpoints (No Changes)
All existing endpoints remain unchanged. Language is client-side only for Phase 3.

---

## Frontend Changes

### 1. App.tsx
- Add i18next initialization
- Add LanguageProvider
- Add language selector button
- Pass translation functions to pages

### 2. New Component: LanguageSelector
```tsx
<LanguageSelector 
  currentLanguage="en" 
  onLanguageChange={(lang) => changeLanguage(lang)}
/>
```

### 3. Updated Components
All pages (Inventory.tsx, POS.tsx) use translation keys:
```tsx
// Before
<h1>Product Inventory</h1>

// After
<h1>{t('inventory.products')}</h1>
```

### 4. Settings Page
Add language preference setting (in Settings component, created in Phase 3)

---

## Translation Keys by Module

### Common Keys
- Buttons: save, cancel, delete, edit, add, close, refresh
- States: loading, error, success, warning, empty
- Actions: create, update, archive, reactivate

### Inventory Keys
- Pages: products, balances, movements, lowStock
- Fields: productName, sku, unit, quantity, onHand
- Actions: addProduct, editProduct, archiveProduct
- Messages: insufficientStock, negativeStock, duplicateSku

### POS Keys
- Pages: pointOfSale, checkout, receipt
- Cart: items, subtotal, discount, tax, total
- Payments: paymentMethod, cash, mobileMoney, card
- Messages: cartEmpty, checkoutSuccess, paymentPending

### Settings Keys
- Sections: profile, language, business, security
- Fields: username, displayName, email, language, currency
- Messages: settingsSaved, settingsError

---

## Formatting Rules

### Numbers (European Style for French)
```
English: 1,234.56
French:  1 234,56

// Format function
const formatNumber = (value, locale) => {
  return new Intl.NumberFormat(locale).format(value);
};
```

### Dates
```
English: 11/15/2026
French:  15/11/2026

// Format function
const formatDate = (date, locale) => {
  return new Intl.DateTimeFormat(locale).format(date);
};
```

### Currency
```
English: XAF 2,500
French:  2 500 XAF

// Format function
const formatCurrency = (value, locale) => {
  return new Intl.NumberFormat(locale, {
    style: 'currency',
    currency: 'XAF'
  }).format(value);
};
```

---

## Dependencies

### Required
```json
{
  "i18next": "^23.7.6",
  "react-i18next": "^13.5.0",
  "i18next-browser-languagedetector": "^7.2.0"
}
```

### Optional
```json
{
  "i18next-http-loader": "^2.4.2",
  "i18next-sprintf-postprocessor": "^0.2.2"
}
```

---

## Testing Strategy

### Unit Tests
- Translation key existence
- Formatting functions
- Language persistence

### Integration Tests
- Language switching (no page reload)
- All UI text translated
- Correct formatting applied
- Settings persistence

### Manual Tests
- Switch between EN/FR
- Verify all pages translated
- Check date/number formatting
- Verify numbers/currencies formatted correctly
- Close/reopen app, verify language restored

---

## Implementation Steps

### Step 1: Setup i18next (1 hour)
- Install dependencies
- Create translation files (en.json, fr.json)
- Initialize i18next in App.tsx
- Create useTranslation hook wrapper

### Step 2: Translate Existing UI (4 hours)
- Inventory page
- POS page
- Navigation
- Settings placeholder

### Step 3: Create Language Selector (1 hour)
- Add to Settings (create Settings page stub)
- Add to top navigation menu
- Implement localStorage persistence

### Step 4: Implement Formatting (2 hours)
- Number formatting
- Date formatting
- Currency formatting
- Receipt formatting

### Step 5: Testing & Polish (2 hours)
- Test language switching
- Verify all text translated
- Check formatting
- Performance optimization

**Total Estimate**: 10 hours

---

## Success Criteria

✅ All UI text translatable (no hardcoded strings)  
✅ Language toggle available (settings + menu)  
✅ Language persists on reload  
✅ Numbers formatted per locale  
✅ Dates formatted per locale  
✅ Currency formatted per locale  
✅ No page reload on language change  
✅ Both EN and FR fully functional  
✅ Zero TypeScript errors  
✅ All tests pass  

---

## Known Scope Exclusions (Phase 3)
- RTL support (not needed for EN/FR)
- Backend message translation (Phase 6+)
- Receipt printing in both languages (Phase 7)
- Email templates (Phase 8+)
- PDF exports (Phase 7)

---

## Files to Create/Modify

### New Files
- `public/locales/en/translation.json` — English translations
- `public/locales/fr/translation.json` — French translations
- `apps/desktop/src/i18n/config.ts` — i18next configuration
- `apps/desktop/src/components/LanguageSelector.tsx` — Language switcher
- `apps/desktop/src/pages/Settings.tsx` — Settings page (new)

### Modified Files
- `apps/desktop/src/App.tsx` — i18next init, language provider
- `apps/desktop/src/pages/Inventory.tsx` — Replace strings with keys
- `apps/desktop/src/pages/POS.tsx` — Replace strings with keys
- `apps/desktop/package.json` — Add i18next dependencies

---

## Next Steps After Phase 3
- Phase 4: Procurement (purchase orders, goods receipt)
- Phase 5: Production (recipes, batches, consumption)
- Phase 6: Cash management (reconciliation, variance)
- Phase 7: Reports & business intelligence

---

**Phase 3 is ready to begin. Estimated completion: 1 week (~30-40 hours with thorough translation and testing)**
