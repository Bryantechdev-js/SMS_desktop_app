import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  Search,
  Plus,
  Package,
  AlertCircle,
  ChevronDown,
  Edit2,
  Archive,
  RotateCcw,
  Eye,
  Trash2,
  TrendingUp,
  Clock,
} from "lucide-react";
import { api, type ProductDetail, type Balance, type StockMovement } from "../lib/api";

export interface InventoryPageProps {
  onError: (msg: string) => void;
}

type Tab = "products" | "balances" | "movements" | "low-stock";

export function InventoryPage({ onError }: InventoryPageProps) {
  const { t } = useTranslation();
  const [tab, setTab] = useState<Tab>("balances");
  const [products, setProducts] = useState<ProductDetail[]>([]);
  const [balances, setBalances] = useState<Balance[]>([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState<"create" | "edit" | null>(null);
  const [selectedProduct, setSelectedProduct] = useState<ProductDetail | null>(null);

  // Form state
  const [name, setName] = useState("");
  const [sku, setSku] = useState("");
  const [description, setDescription] = useState("");
  const [category, setCategory] = useState("Bakery");
  const [unit, setUnit] = useState("piece");
  const [sellingPrice, setSellingPrice] = useState("0");
  const [purchasePrice, setPurchasePrice] = useState("");
  const [taxRate, setTaxRate] = useState("0");
  const [productType, setProductType] = useState("finished_product");
  const [minThreshold, setMinThreshold] = useState("");
  const [reorderQty, setReorderQty] = useState("");
  const [barcode, setBarcode] = useState("");
  const [storageLocation, setStorageLocation] = useState("");
  const [isPerishable, setIsPerishable] = useState(false);
  const [busy, setBusy] = useState(false);

  async function loadData() {
    try {
      setLoading(true);
      const [prods, bals] = await Promise.all([
        api.products(),
        api.balances(),
      ]);
      setProducts(prods as any);
      setBalances(bals);
    } catch (err) {
      onError(
        err instanceof Error ? err.message : "Failed to load inventory data"
      );
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadData();
  }, []);

  const filtered = useMemo(
    () =>
      products.filter(
        (p) =>
          `${p.name} ${p.sku} ${p.category}`
            .toLowerCase()
            .includes(query.toLowerCase()) && p.active
      ),
    [products, query]
  );

  const lowStockItems = useMemo(
    () =>
      balances.filter(
        (b) =>
          products.find(
            (p) =>
              p.id === b.product_id &&
              p.minimum_stock_threshold &&
              Number(b.quantity) <= Number(p.minimum_stock_threshold)
          )
      ),
    [balances, products]
  );

  const resetForm = () => {
    setName("");
    setSku("");
    setDescription("");
    setCategory("Bakery");
    setUnit("piece");
    setSellingPrice("0");
    setPurchasePrice("");
    setTaxRate("0");
    setProductType("finished_product");
    setMinThreshold("");
    setReorderQty("");
    setBarcode("");
    setStorageLocation("");
    setIsPerishable(false);
    setSelectedProduct(null);
  };

  async function handleSave() {
    setBusy(true);
    try {
      if (showModal === "create") {
        await api.createProduct({
          name: name.trim(),
          sku: sku.trim().toUpperCase(),
          category: category.trim(),
          unit: unit.trim(),
          selling_price: sellingPrice,
        } as any);
        resetForm();
      } else if (showModal === "edit" && selectedProduct) {
        await api.updateProduct(selectedProduct.id, {
          name: name.trim(),
          description: description.trim(),
          category: category.trim(),
          unit: unit.trim(),
          selling_price: sellingPrice,
          purchase_price: purchasePrice ? purchasePrice : undefined,
          tax_rate: taxRate,
          product_type: productType,
          minimum_stock_threshold: minThreshold ? minThreshold : undefined,
          reorder_quantity: reorderQty ? reorderQty : undefined,
          barcode: barcode.trim() || undefined,
          storage_location: storageLocation.trim() || undefined,
          is_perishable: isPerishable,
        } as any);
      }
      setShowModal(null);
      await loadData();
    } catch (err) {
      onError(err instanceof Error ? err.message : "Failed to save product");
    } finally {
      setBusy(false);
    }
  }

  async function handleArchive(productId: string) {
    if (!confirm("Archive this product?")) return;
    try {
      await api.archiveProduct(productId, "Archived by user");
      await loadData();
    } catch (err) {
      onError(err instanceof Error ? err.message : "Failed to archive product");
    }
  }

  const openEditModal = (product: ProductDetail) => {
    setSelectedProduct(product);
    setName(product.name);
    setSku(product.sku);
    setDescription(product.description);
    setCategory(product.category);
    setUnit(product.unit);
    setSellingPrice(product.selling_price);
    setPurchasePrice(product.purchase_price || "");
    setTaxRate(product.tax_rate);
    setProductType(product.product_type);
    setMinThreshold(product.minimum_stock_threshold || "");
    setReorderQty(product.reorder_quantity || "");
    setBarcode(product.barcode || "");
    setStorageLocation(product.storage_location || "");
    setIsPerishable(product.is_perishable);
    setShowModal("edit");
  };

  return (
    <div className="inventory-page">
      <div className="page-heading">
        <div>
          <div className="eyebrow">{t("inventory.title").toUpperCase()}</div>
          <h1>{t("inventory.products")} & {t("inventory.balances")}</h1>
          <p>{t("inventory.title")}</p>
        </div>
        <div className="heading-actions">
          <button
            className="btn btn-primary"
            onClick={() => {
              resetForm();
              setShowModal("create");
            }}
          >
            <Plus size={16} /> {t("inventory.addProduct")}
          </button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="tabs-nav">
        <button
          className={`tab ${tab === "balances" ? "active" : ""}`}
          onClick={() => setTab("balances")}
        >
          <Package size={16} /> {t("inventory.balances")}
        </button>
        <button
          className={`tab ${tab === "products" ? "active" : ""}`}
          onClick={() => setTab("products")}
        >
          <TrendingUp size={16} /> {t("inventory.products")}
        </button>
        <button
          className={`tab ${tab === "low-stock" ? "active" : ""}`}
          onClick={() => setTab("low-stock")}
        >
          <AlertCircle size={16} /> {t("inventory.lowStockAlert")} ({lowStockItems.length})
        </button>
        <button
          className={`tab ${tab === "movements" ? "active" : ""}`}
          onClick={() => setTab("movements")}
        >
          <Clock size={16} /> {t("inventory.movements")}
        </button>
      </div>

      {/* Content */}
      <div className="panel">
        {tab === "balances" && (
          <>
            <div className="panel-head">
              <div>
                <h2>{t("inventory.balances")}</h2>
                <p>{t("dashboard.currentBalances")}</p>
              </div>
              <div className="searchbox">
                <Search size={16} />
                <input
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder={t("common.search")}
                />
              </div>
            </div>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>{t("inventory.productName")}</th>
                    <th>{t("inventory.sku")}</th>
                    <th>{t("inventory.unit")}</th>
                    <th>{t("inventory.onHand")}</th>
                    <th>{t("common.status")}</th>
                    <th>{t("common.actions")}</th>
                  </tr>
                </thead>
                <tbody>
                  {loading ? (
                    <tr>
                      <td colSpan={6} className="empty">
                        {t("common.loading")}
                      </td>
                    </tr>
                  ) : balances.length === 0 ? (
                    <tr>
                      <td colSpan={6} className="empty">
                        {t("common.empty")}
                      </td>
                    </tr>
                  ) : (
                    balances.map((b) => {
                      const product = products.find((p) => p.id === b.product_id);
                      const isLow =
                        product?.minimum_stock_threshold &&
                        Number(b.quantity) <= Number(product.minimum_stock_threshold);
                      return (
                        <tr key={b.product_id}>
                          <td>
                            <b>{b.name}</b>
                          </td>
                          <td className="muted">{b.sku}</td>
                          <td>{b.unit}</td>
                          <td className="quantity">
                            {Number(b.quantity).toLocaleString()}
                          </td>
                          <td>
                            <span className={`status ${isLow ? "low" : "healthy"}`}>
                              {isLow ? t("inventory.lowStockAlert") : t("inventory.inStock")}
                            </span>
                          </td>
                          <td>
                            <button
                              className="icon-text-btn"
                              onClick={() =>
                                openEditModal(
                                  products.find((p) => p.id === b.product_id)!
                                )
                              }
                            >
                              <Edit2 size={14} /> {t("common.edit")}
                            </button>
                          </td>
                        </tr>
                      );
                    })
                  )}
                </tbody>
              </table>
            </div>
          </>
        )}

        {tab === "products" && (
          <>
            <div className="panel-head">
              <div>
                <h2>{t("inventory.products")}</h2>
                <p>{t("inventory.title")}</p>
              </div>
              <div className="searchbox">
                <Search size={16} />
                <input
                  value={query}
                  onChange={(e) => setQuery(e.target.value)}
                  placeholder={t("common.search")}
                />
              </div>
            </div>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>{t("inventory.productName")}</th>
                    <th>{t("inventory.sku")}</th>
                    <th>{t("inventory.productType")}</th>
                    <th>{t("inventory.sellingPrice")}</th>
                    <th>{t("inventory.taxRate")}</th>
                    <th>{t("common.actions")}</th>
                  </tr>
                </thead>
                <tbody>
                  {loading ? (
                    <tr>
                      <td colSpan={6} className="empty">
                        {t("common.loading")}
                      </td>
                    </tr>
                  ) : filtered.length === 0 ? (
                    <tr>
                      <td colSpan={6} className="empty">
                        {t("common.empty")}
                      </td>
                    </tr>
                  ) : (
                    filtered.map((p) => (
                      <tr key={p.id}>
                        <td>
                          <div>
                            <b>{p.name}</b>
                            <small>{p.description}</small>
                          </div>
                        </td>
                        <td className="muted">{p.sku}</td>
                        <td>
                          <span className="badge">{p.product_type.replace(/_/g, " ")}</span>
                        </td>
                        <td>{Number(p.selling_price).toLocaleString()}</td>
                        <td>{Number(p.tax_rate).toFixed(1)}%</td>
                        <td>
                          <div className="action-group">
                            <button
                              className="icon-text-btn"
                              onClick={() => openEditModal(p)}
                            >
                              <Edit2 size={14} /> {t("common.edit")}
                            </button>
                            <button
                              className="icon-text-btn danger"
                              onClick={() => handleArchive(p.id)}
                            >
                              <Archive size={14} /> {t("common.archive")}
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </>
        )}

        {tab === "low-stock" && (
          <>
            <div className="panel-head">
              <div>
                <h2>{t("inventory.lowStockAlert")}</h2>
                <p>{t("inventory.minimumThreshold")}</p>
              </div>
            </div>
            <div className="table-wrap">
              <table>
                <thead>
                  <tr>
                    <th>{t("inventory.productName")}</th>
                    <th>{t("inventory.sku")}</th>
                    <th>{t("inventory.onHand")}</th>
                    <th>{t("inventory.minimumThreshold")}</th>
                    <th>{t("inventory.reorderQuantity")}</th>
                  </tr>
                </thead>
                <tbody>
                  {lowStockItems.length === 0 ? (
                    <tr>
                      <td colSpan={5} className="empty">
                        {t("common.empty")}
                      </td>
                    </tr>
                  ) : (
                    lowStockItems.map((b) => {
                      const product = products.find((p) => p.id === b.product_id)!;
                      return (
                        <tr key={b.product_id}>
                          <td>
                            <b>{b.name}</b>
                          </td>
                          <td className="muted">{b.sku}</td>
                          <td className="quantity warn">
                            {Number(b.quantity).toLocaleString()}
                          </td>
                          <td>{Number(product.minimum_stock_threshold || 0).toLocaleString()}</td>
                          <td>{Number(product.reorder_quantity || 0).toLocaleString()}</td>
                        </tr>
                      );
                    })
                  )}
                </tbody>
              </table>
            </div>
          </>
        )}

        {tab === "movements" && (
          <>
            <div className="panel-head">
              <div>
                <h2>{t("inventory.movements")}</h2>
                <p>{t("inventory.title")}</p>
              </div>
            </div>
            <p className="muted" style={{ padding: "16px" }}>
              Stock movement history is logged in the database and can be viewed by filtering on specific
              products via the API. This view is coming in the next iteration.
            </p>
          </>
        )}
      </div>

      {/* Modal: Create/Edit Product */}
      {showModal && (
        <div
          className="modal-backdrop"
          onMouseDown={(e) => {
            if (e.target === e.currentTarget) setShowModal(null);
          }}
        >
          <form
            className="modal modal-lg"
            onSubmit={(e) => {
              e.preventDefault();
              void handleSave();
            }}
          >
            <div className="modal-head">
              <div>
                <h2>{showModal === "create" ? t("inventory.addProduct") : t("common.edit")}</h2>
                <p>
                  {showModal === "create"
                    ? "Create a new catalog item."
                    : "Update product information."}
                </p>
              </div>
              <button type="button" className="close" onClick={() => setShowModal(null)}>
                ×
              </button>
            </div>

            <div className="modal-scroll">
              {/* Basic Info */}
              <div className="form-section">
                <h3>{t("common.details")}</h3>
                <label>
                  {t("inventory.productName")} *
                  <input
                    required
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                    placeholder="e.g. Butter Croissant"
                  />
                </label>
                <label>
                  {t("inventory.sku")} *
                  <input
                    required
                    value={sku}
                    onChange={(e) => setSku(e.target.value)}
                    placeholder="e.g. BAK-001"
                    disabled={showModal === "edit"}
                  />
                </label>
                <label>
                  {t("inventory.description")}
                  <textarea
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    placeholder="Product details and notes"
                    rows={2}
                  />
                </label>
              </div>

              {/* Classification */}
              <div className="form-section">
                <h3>{t("inventory.category")}</h3>
                <div className="form-row">
                  <label>
                    {t("inventory.category")}
                    <input
                      value={category}
                      onChange={(e) => setCategory(e.target.value)}
                      placeholder="e.g. Bakery"
                    />
                  </label>
                  <label>
                    {t("inventory.productType")}
                    <select
                      value={productType}
                      onChange={(e) => setProductType(e.target.value)}
                    >
                      <option value="ingredient">Ingredient</option>
                      <option value="finished_product">Finished Product</option>
                      <option value="packaging">Packaging</option>
                      <option value="other">Other</option>
                    </select>
                  </label>
                </div>
                <label>
                  <input
                    type="checkbox"
                    checked={isPerishable}
                    onChange={(e) => setIsPerishable(e.target.checked)}
                  />
                  {t("inventory.perishable")}
                </label>
              </div>

              {/* Pricing */}
              <div className="form-section">
                <h3>{t("inventory.sellingPrice")}</h3>
                <div className="form-row">
                  <label>
                    {t("inventory.unit")} *
                    <input
                      required
                      value={unit}
                      onChange={(e) => setUnit(e.target.value)}
                      placeholder="e.g. piece, kg, liter"
                    />
                  </label>
                  <label>
                    {t("inventory.sellingPrice")} *
                    <input
                      required
                      type="number"
                      step="0.01"
                      min="0"
                      value={sellingPrice}
                      onChange={(e) => setSellingPrice(e.target.value)}
                    />
                  </label>
                </div>
                <div className="form-row">
                  <label>
                    {t("inventory.purchasePrice")}
                    <input
                      type="number"
                      step="0.01"
                      min="0"
                      value={purchasePrice}
                      onChange={(e) => setPurchasePrice(e.target.value)}
                      placeholder="Optional"
                    />
                  </label>
                  <label>
                    {t("inventory.taxRate")}
                    <input
                      type="number"
                      step="0.1"
                      min="0"
                      max="100"
                      value={taxRate}
                      onChange={(e) => setTaxRate(e.target.value)}
                    />
                  </label>
                </div>
              </div>

              {/* Inventory */}
              <div className="form-section">
                <h3>{t("inventory.balances")}</h3>
                <div className="form-row">
                  <label>
                    {t("inventory.minimumThreshold")}
                    <input
                      type="number"
                      step="0.01"
                      min="0"
                      value={minThreshold}
                      onChange={(e) => setMinThreshold(e.target.value)}
                      placeholder="Trigger low-stock alert"
                    />
                  </label>
                  <label>
                    {t("inventory.reorderQuantity")}
                    <input
                      type="number"
                      step="0.01"
                      min="0"
                      value={reorderQty}
                      onChange={(e) => setReorderQty(e.target.value)}
                      placeholder="Amount to order"
                    />
                  </label>
                </div>
                <div className="form-row">
                  <label>
                    {t("inventory.barcode")}
                    <input
                      value={barcode}
                      onChange={(e) => setBarcode(e.target.value)}
                      placeholder="Optional barcode"
                    />
                  </label>
                  <label>
                    {t("inventory.storageLocation")}
                    <input
                      value={storageLocation}
                      onChange={(e) => setStorageLocation(e.target.value)}
                      placeholder="e.g. Shelf A1"
                    />
                  </label>
                </div>
              </div>
            </div>

            <div className="modal-actions">
              <button type="button" className="btn btn-secondary" onClick={() => setShowModal(null)}>
                {t("common.cancel")}
              </button>
              <button type="submit" className="btn btn-primary" disabled={busy}>
                {busy ? `${t("common.loading")}` : showModal === "create" ? t("inventory.addProduct") : t("common.update")}
              </button>
            </div>
          </form>
        </div>
      )}

      <style>{`
        .inventory-page { display: flex; flex-direction: column; gap: 24px; }
        .tabs-nav {
          display: flex;
          gap: 12px;
          border-bottom: 1px solid var(--color-border);
          overflow-x: auto;
        }
        .tab {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 12px 16px;
          border: none;
          background: transparent;
          color: var(--color-text-secondary);
          cursor: pointer;
          border-bottom: 2px solid transparent;
          transition: all 0.2s;
          font-size: 0.875rem;
          font-weight: 500;
          white-space: nowrap;
        }
        .tab:hover { color: var(--color-text); }
        .tab.active {
          color: var(--color-primary);
          border-bottom-color: var(--color-primary);
        }
        .form-section { margin-bottom: 24px; }
        .form-section h3 {
          font-size: 0.875rem;
          font-weight: 600;
          color: var(--color-text-secondary);
          text-transform: uppercase;
          margin-bottom: 12px;
          letter-spacing: 0.5px;
        }
        .modal-scroll {
          max-height: 60vh;
          overflow-y: auto;
          padding: 24px;
        }
        .action-group {
          display: flex;
          gap: 8px;
        }
        .icon-text-btn {
          display: flex;
          align-items: center;
          gap: 6px;
          padding: 6px 12px;
          border: 1px solid var(--color-border);
          background: transparent;
          color: var(--color-text);
          cursor: pointer;
          border-radius: 6px;
          font-size: 0.875rem;
          transition: all 0.2s;
        }
        .icon-text-btn:hover {
          background: var(--color-hover);
          border-color: var(--color-text);
        }
        .icon-text-btn.danger:hover {
          background: rgba(220, 53, 69, 0.1);
          border-color: #dc3545;
          color: #dc3545;
        }
        .badge {
          display: inline-block;
          padding: 4px 8px;
          background: var(--color-hover);
          border-radius: 4px;
          font-size: 0.75rem;
          text-transform: uppercase;
          font-weight: 600;
          color: var(--color-text-secondary);
        }
      `}</style>
    </div>
  );
}
