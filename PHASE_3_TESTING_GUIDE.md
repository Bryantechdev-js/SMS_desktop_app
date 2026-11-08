# Phase 3 Testing Guide - English/French Internationalization

**Purpose**: Verify that all i18n functionality works correctly in both English and French

**Prerequisites**:
- Dependencies installed: `npm install` (in `apps/desktop`)
- Development server running: `npm run dev`
- Both Inventory and POS pages accessible
- Settings page accessible

---

## Test Cases

### Test 1: Language Selector Visibility ✅

**Steps**:
1. Open the desktop app and log in
2. Look at the top navigation bar (topbar)
3. Find the globe icon with "EN" next to it

**Expected Result**:
- Globe icon visible on right side of topbar
- Current language code displayed (EN or FR)
- Clickable button

**Pass/Fail**: ___

---

### Test 2: Navigate to Settings

**Steps**:
1. Click the globe button (EN/FR) in topbar
2. Verify page changes to Settings

**Expected Result**:
- Settings page loads
- No errors in browser console
- Settings title displays translated

**Pass/Fail**: ___

---

### Test 3: Language Selector in Settings

**Steps**:
1. On Settings page, find "Language" preference section
2. Verify LanguageSelector component is visible
3. Should show EN and FR buttons

**Expected Result**:
- Two language buttons visible (EN, FR)
- Current language button appears highlighted/active
- Both buttons are clickable

**Pass/Fail**: ___

---

### Test 4: Switch to French

**Steps**:
1. In Settings Language section, click FR button
2. Observe page changes

**Expected Result**:
- No page reload (smooth transition)
- All text changes to French
- "EN" button in topbar changes to "FR"
- No errors in console

**Verify translations** (Sample):
- "Settings" → "Paramètres" ✓
- "Language" → "Langue" ✓
- "Business Information" → "Informations commerciales" ✓

**Pass/Fail**: ___

---

### Test 5: Switch Back to English

**Steps**:
1. Click EN button in Settings Language section
2. Observe page changes back

**Expected Result**:
- Instant language switch to English
- All text in English
- "FR" button in topbar changes back to "EN"
- No page reload

**Pass/Fail**: ___

---

### Test 6: Language Persistence (Reload)

**Steps**:
1. Switch to French (FR)
2. Close the app entirely (not just refresh)
3. Reopen the app and log in again
4. Check the language

**Expected Result**:
- App opens in French (language persisted)
- No manual language selection needed
- Proof: Language preference stored in localStorage

**Pass/Fail**: ___

---

### Test 7: Inventory Page - English

**Steps**:
1. Make sure language is EN (English)
2. Click "Inventory" in navigation
3. Verify all text is in English

**Expected Result - Tab Navigation**:
- "Stock Balances" (not "Soldes des stocks")
- "Products" (not "Produits")
- "Low Stock Alert" (not "Alerte de stock faible")
- "Movements" (not "Mouvements")

**Expected Result - Table Headers**:
- "PRODUCT" (not "PRODUIT")
- "SKU" (not "SKU")
- "UNIT" (not "UNITÉ")
- "ON-HAND" (not "EN MAIN")
- "STATUS" (not "STATUT")
- "ACTIONS" (not "ACTIONS")

**Expected Result - Buttons**:
- "Add product" visible
- When adding: "Create product" button (not "Créer un produit")

**Pass/Fail**: ___

---

### Test 8: Inventory Page - French

**Steps**:
1. Switch to French (FR) in Settings
2. Navigate to Inventory page (or stay if already there)
3. Verify all text is in French

**Expected Result - Tab Navigation**:
- "Soldes des stocks" ✓
- "Produits" ✓
- "Alerte de stock faible" ✓
- "Mouvements" ✓

**Expected Result - Table Headers**:
- "PRODUIT" ✓
- "SKU" ✓
- "UNITÉ" ✓
- "EN MAIN" ✓
- "STATUT" ✓
- "ACTIONS" ✓

**Expected Result - Buttons**:
- "Ajouter un produit" visible
- When adding: "Créer un produit" button

**Expected Result - Status Indicators**:
- "En stock" (not "In stock") ✓
- "Stock faible" (not "Low stock") ✓

**Pass/Fail**: ___

---

### Test 9: POS Page - English

**Steps**:
1. Switch to English (EN) if not already
2. Click "POS" or "Point of Sale" in navigation
3. Verify all POS text is in English

**Expected Result - Page**:
- "Point of Sale" title (not "Point de vente")
- Search box placeholder: "Search products"

**Expected Result - Cart Section**:
- "Cart" label
- Empty state: "Add products to cart"
- "Subtotal" label
- "Discount" label
- "Total" label
- "Payment Method" label

**Expected Result - Payment Method Options**:
- "Cash" ✓
- "Mobile Money" ✓
- "Card" ✓
- "Bank Transfer" ✓

**Expected Result - Button**:
- "Complete Sale" (not "Finaliser la vente")

**Pass/Fail**: ___

---

### Test 10: POS Page - French

**Steps**:
1. Switch to French (FR) in Settings
2. Navigate to POS page (or stay if already there)
3. Verify all POS text is in French

**Expected Result - Page**:
- "Point de vente" title ✓
- Search box placeholder: "Rechercher des produits"

**Expected Result - Cart Section**:
- "Panier" label ✓
- Empty state: "Ajoutez des produits au panier"
- "Sous-total" label ✓
- "Remise" label ✓
- "Total" label ✓
- "Méthode de paiement" label ✓

**Expected Result - Payment Method Options**:
- "Espèces" ✓
- "Argent mobile" ✓
- "Carte" ✓
- "Virement bancaire" ✓

**Expected Result - Button**:
- "Finaliser la vente" ✓

**Pass/Fail**: ___

---

### Test 11: Receipt Modal - Both Languages

**Steps**:
1. On POS page, add a product to cart
2. Set payment amount
3. Complete the sale (submit checkout)
4. Verify receipt modal appears with correct language

**English** (if EN selected):
- "Sale Complete ✓" title
- "Ticket #" label
- "Total:" label
- "Payment:" label
- "Start New Sale" button

**French** (if FR selected):
- "Vente complétée ✓" title
- "Ticket #" label
- "Total :" label
- "Statut du paiement :" label
- "Nouvelle vente" button

**Pass/Fail**: ___

---

### Test 12: Navigation Labels - Both Languages

**Steps**:
1. Switch to English (EN)
2. Look at left sidebar navigation
3. Verify labels

**English**:
- "Overview" ✓
- "Inventory" ✓
- "Point of Sale" ✓
- "Production" ✓
- "Procurement" ✓
- "Cash" ✓
- "Settings" ✓

**Steps - French**:
1. Switch to French (FR)
2. Look at left sidebar navigation again

**French**:
- "Aperçu" ✓
- "Inventaire" ✓
- "Point de vente" ✓
- "Production" ✓
- "Approvisionnement" ✓
- "Caisse" ✓
- "Paramètres" ✓

**Pass/Fail**: ___

---

### Test 13: Dashboard Stats - Both Languages

**Steps**:
1. Switch to English (EN)
2. Go to Overview page
3. Verify stat labels

**English**:
- "PRODUCTS IN CATALOG" ✓
- "STOCK UNITS" ✓
- "LOW STOCK ITEMS" ✓
- "API STATUS" ✓

**Steps - French**:
1. Switch to French (FR)
2. Observe stats section

**French**:
- "PRODUITS AU CATALOGUE" ✓
- "UNITÉS DE STOCK" ✓
- "ARTICLES À STOCK FAIBLE" ✓
- "STATUT DE L'API" ✓

**Pass/Fail**: ___

---

### Test 14: Auth Page - Both Languages

**Steps**:
1. Log out (click avatar in topbar, then Sign out)
2. Verify login/setup screen

**English**:
- "Welcome back" title
- "Username" label
- "Password" label
- "Sign in securely" button

**Steps - Switch Language on Auth Page**:
1. On login screen, cannot directly switch language
2. Use browser console to test: `localStorage.setItem('language', 'fr'); location.reload()`
3. Verify page reloads with French

**French**:
- "Bienvenue" title ✓
- "Nom d'utilisateur" label ✓
- "Mot de passe" label ✓
- "Se connecter de manière sécurisée" button ✓

**Pass/Fail**: ___

---

### Test 15: Settings Form Labels - Both Languages

**Steps**:
1. Go to Settings page
2. Switch to English (EN)
3. Verify form labels

**English**:
- "Business Information" heading
- "Business name" label
- "Address" label
- "Phone" label
- "Email" label
- "Preferences" heading
- "Language" label
- "Currency" label

**Steps - French**:
1. Switch to French (FR)
2. Observe Settings form

**French**:
- "Informations commerciales" ✓
- "Nom de l'entreprise" ✓
- "Adresse" ✓
- "Téléphone" ✓
- "E-mail" ✓
- "Préférences" ✓
- "Langue" ✓
- "Devise" ✓

**Pass/Fail**: ___

---

### Test 16: No Page Reloads on Language Switch

**Steps**:
1. Open browser DevTools (F12)
2. Go to Network tab
3. Clear network log
4. Switch language from EN to FR
5. Check if page reloads

**Expected Result**:
- No page reload occurs
- No new requests to HTML file
- No full page flicker
- Language switches instantly
- Network tab remains empty (no document reload)

**Pass/Fail**: ___

---

### Test 17: Browser Console - No Errors

**Steps**:
1. Open browser DevTools (F12)
2. Click Console tab
3. Perform language switches
4. Navigate between pages
5. Check for any errors or warnings

**Expected Result**:
- No errors in console
- No warnings related to i18next
- No module loading errors
- Clean console

**Pass/Fail**: ___

---

### Test 18: localStorage Persistence

**Steps**:
1. Open browser DevTools (F12)
2. Go to Application or Storage tab
3. Find localStorage for the app domain
4. Look for "language" key

**English**:
```
language: "en"
```

**French**:
```
language: "fr"
```

**Steps - Verify Persistence**:
1. Switch language to FR
2. Check localStorage shows "language: fr"
3. Manually clear localStorage
4. Refresh page
5. Language should revert to default (EN)
6. Switch to FR again
7. Verify localStorage shows "language: fr"
8. Close tab, reopen
9. Language should still be FR

**Pass/Fail**: ___

---

### Test 19: Complete User Flow - English

**Steps**:
1. Ensure language is EN
2. Go to Inventory
3. Search for a product
4. Note all text is English
5. Go to POS
6. Add product to cart
7. Complete sale
8. Verify receipt in English
9. Return to Overview
10. All navigation and stats in English

**Pass/Fail**: ___

---

### Test 20: Complete User Flow - French

**Steps**:
1. Switch to FR
2. Go to Inventory
3. Search for a product
4. Note all text is French
5. Go to POS
6. Add product to cart
7. Complete sale
8. Verify receipt in French
9. Return to Overview
10. All navigation and stats in French

**Pass/Fail**: ___

---

## Summary Checklist

- [ ] All 20 tests passed
- [ ] No console errors
- [ ] No TypeScript errors
- [ ] Language persistence works
- [ ] No page reloads on switch
- [ ] All UI text translated
- [ ] Both English and French complete
- [ ] Settings page works correctly
- [ ] Language selector accessible
- [ ] Ready for production

---

## Known Limitations

1. **Backend messages**: Not translated (future phase)
2. **Error messages from API**: Show English only (future enhancement)
3. **Receipt printing**: Uses current language (no multi-language receipt)
4. **Email templates**: Not i18n enabled (future phase)

---

## Bug Report Template

If you find an issue during testing:

```
### Issue Title
[Describe the issue]

### Steps to Reproduce
1. [Step 1]
2. [Step 2]
3. [Step 3]

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happens]

### Language
- [ ] English (EN)
- [ ] French (FR)

### Page
- [ ] Overview
- [ ] Inventory
- [ ] POS
- [ ] Settings
- [ ] Auth

### Screenshots
[If applicable]

### Browser Console
[Copy any errors]
```

---

**Testing Complete!**  
All test cases should pass with 100% success rate.
