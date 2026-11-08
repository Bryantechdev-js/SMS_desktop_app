import { useState, useEffect, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  Search,
  ShoppingCart as CartIcon,
  Trash2,
} from "lucide-react";
import {
  api,
  type Product,
  type CheckoutLineItem,
  type CheckoutResponse,
} from "../lib/api";

export interface POSPageProps {
  onError: (msg: string) => void;
}

export function POSPage({ onError }: POSPageProps) {
  const { t } = useTranslation();
  const [products, setProducts] = useState<Product[]>([]);
  const [loading, setLoading] = useState(true);
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("");

  // Cart state
  const [cart, setCart] = useState<
    (CheckoutLineItem & { product: Product; temp_id: string })[]
  >([]);
  const [discount, setDiscount] = useState("0");
  const [paymentMethod, setPaymentMethod] = useState("cash");
  const [paymentAmount, setPaymentAmount] = useState("");
  const [busy, setBusy] = useState(false);
  const [receipt, setReceipt] = useState<CheckoutResponse | null>(null);

  useEffect(() => {
    async function load() {
      try {
        const prods = await api.products();
        setProducts(prods as any);
      } catch (err) {
        onError(err instanceof Error ? err.message : "Failed to load products");
      } finally {
        setLoading(false);
      }
    }
    void load();
  }, []);

  const filtered = useMemo(
    () =>
      products.filter(
        (p) =>
          p.active &&
          `${p.name} ${p.sku} ${p.category}`
            .toLowerCase()
            .includes(query.toLowerCase()) &&
          (category ? p.category === category : true)
      ),
    [products, query, category]
  );

  const categories = useMemo(
    () => [...new Set(products.map((p) => p.category))],
    [products]
  );

  const subtotal = useMemo(
    () =>
      cart.reduce(
        (sum, item) =>
          sum + (Number(item.unit_price) * Number(item.quantity) -
            Number(item.discount_amount)),
        0
      ),
    [cart]
  );

  const discountAmount = useMemo(
    () => Math.min(Number(discount), subtotal),
    [discount, subtotal]
  );

  const tax = 0; // Simplified for Phase 2
  const total = subtotal - discountAmount + tax;

  const addToCart = (product: Product) => {
    const existing = cart.find((item) => item.product_id === product.id);
    if (existing) {
      setCart(
        cart.map((item) =>
          item.product_id === product.id
            ? {
                ...item,
                quantity: item.quantity + 1,
              }
            : item
        )
      );
    } else {
      setCart([
        ...cart,
        {
          product_id: product.id,
          quantity: 1,
          unit_price: product.selling_price,
          discount_amount: "0",
          product,
          temp_id: Math.random().toString(),
        },
      ]);
    }
  };

  const updateQuantity = (tempId: string, qty: number) => {
    if (qty <= 0) {
      setCart(cart.filter((item) => item.temp_id !== tempId));
    } else {
      setCart(
        cart.map((item) =>
          item.temp_id === tempId ? { ...item, quantity: qty } : item
        )
      );
    }
  };

  const removeFromCart = (tempId: string) => {
    setCart(cart.filter((item) => item.temp_id !== tempId));
  };

  const handleCheckout = async () => {
    if (cart.length === 0) {
      onError("Cart is empty");
      return;
    }

    const payAmt = Number(paymentAmount);
    if (payAmt < total) {
      onError(`Payment ${payAmt} is less than total ${total}`);
      return;
    }

    setBusy(true);
    try {
      const response = await api.checkout({
        items: cart.map((item) => ({
          product_id: item.product_id,
          quantity: item.quantity.toString(),
          unit_price: item.unit_price,
          discount_amount: item.discount_amount,
        })),
        discount_amount: discountAmount.toString(),
        payment_method: paymentMethod,
        payment_amount: payAmt.toString(),
      });
      setReceipt(response);
      setCart([]);
      setDiscount("0");
      setPaymentAmount("");
    } catch (err) {
      onError(err instanceof Error ? err.message : "Checkout failed");
    } finally {
      setBusy(false);
    }
  };

  if (receipt) {
    return (
      <div className="pos-receipt-modal">
        <div className="receipt-content">
          <h2>{t("pos.saleComplete")} ✓</h2>
          <div className="receipt-info">
            <p>
              <strong>{t("pos.ticketNumber")}:</strong> {receipt.ticket_number}
            </p>
            <p>
              <strong>{t("pos.total")}:</strong> XAF {Number(receipt.total_amount).toLocaleString()}
            </p>
            <p>
              <strong>{t("pos.paymentStatus")}:</strong> {receipt.payment_status}
            </p>
          </div>
          <button
            className="btn btn-primary"
            onClick={() => setReceipt(null)}
          >
            {t("pos.startNewSale")}
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="pos-page">
      <div className="pos-header">
        <h1>{t("pos.title")}</h1>
      </div>

      <div className="pos-layout">
        {/* Product Browser */}
        <div className="pos-products">
          <div className="pos-search">
            <Search size={16} />
            <input
              type="text"
              placeholder={t("pos.searchProducts")}
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>

          <div className="pos-categories">
            {categories.map((cat) => (
              <button
                key={cat}
                className={`category-btn ${category === cat ? "active" : ""}`}
                onClick={() => setCategory(category === cat ? "" : cat)}
              >
                {cat}
              </button>
            ))}
          </div>

          <div className="pos-products-grid">
            {loading ? (
              <p>{t("common.loading")}</p>
            ) : filtered.length === 0 ? (
              <p>{t("common.empty")}</p>
            ) : (
              filtered.map((p) => (
                <button
                  key={p.id}
                  className="product-card"
                  onClick={() => addToCart(p)}
                >
                  <div className="product-name">{p.name}</div>
                  <div className="product-sku">{p.sku}</div>
                  <div className="product-price">
                    XAF {Number(p.selling_price).toLocaleString()}
                  </div>
                </button>
              ))
            )}
          </div>
        </div>

        {/* Cart & Checkout */}
        <div className="pos-cart-section">
          <div className="cart-header">
            <CartIcon size={20} />
            <h2>{t("pos.cart")}</h2>
            <span className="cart-count">{cart.length}</span>
          </div>

          <div className="cart-items">
            {cart.length === 0 ? (
              <p className="empty-cart">{t("pos.cartEmpty")}</p>
            ) : (
              cart.map((item) => (
                <div key={item.temp_id} className="cart-item">
                  <div className="item-info">
                    <div className="item-name">{item.product.name}</div>
                    <div className="item-price">
                      XAF {Number(item.unit_price).toLocaleString()}
                    </div>
                  </div>
                  <div className="item-qty">
                    <button
                      onClick={() =>
                        updateQuantity(item.temp_id, item.quantity - 1)
                      }
                    >
                      −
                    </button>
                    <input
                      type="number"
                      value={item.quantity}
                      onChange={(e) =>
                        updateQuantity(
                          item.temp_id,
                          Number(e.target.value)
                        )
                      }
                      min="1"
                    />
                    <button
                      onClick={() =>
                        updateQuantity(item.temp_id, item.quantity + 1)
                      }
                    >
                      +
                    </button>
                  </div>
                  <button
                    className="remove-btn"
                    onClick={() => removeFromCart(item.temp_id)}
                  >
                    <Trash2 size={16} />
                  </button>
                </div>
              ))
            )}
          </div>

          <div className="cart-totals">
            <div className="total-row">
              <span>{t("pos.subtotal")}</span>
              <strong>XAF {subtotal.toLocaleString()}</strong>
            </div>

            <div className="discount-input">
              <label>{t("pos.discount")}</label>
              <input
                type="number"
                value={discount}
                onChange={(e) => setDiscount(e.target.value)}
                min="0"
                max={subtotal}
              />
            </div>

            <div className="total-row highlight">
              <span>{t("pos.total")}</span>
              <strong>XAF {total.toLocaleString()}</strong>
            </div>

            <div className="payment-method">
              <label>{t("pos.paymentMethod")}</label>
              <select
                value={paymentMethod}
                onChange={(e) => setPaymentMethod(e.target.value)}
              >
                <option value="cash">{t("pos.cash")}</option>
                <option value="mobile_money">{t("pos.mobileMoney")}</option>
                <option value="card">{t("pos.card")}</option>
                <option value="bank_transfer">{t("pos.bankTransfer")}</option>
              </select>
            </div>

            <div className="payment-amount">
              <label>{t("pos.amountToPay")}</label>
              <input
                type="number"
                value={paymentAmount}
                onChange={(e) => setPaymentAmount(e.target.value)}
                min={total}
                placeholder={total.toString()}
              />
            </div>

            <button
              className="btn btn-primary btn-full"
              onClick={handleCheckout}
              disabled={busy || cart.length === 0}
            >
              {busy ? t("pos.processing") : t("pos.completeSale")}
            </button>
          </div>
        </div>
      </div>

      <style>{`
        .pos-page {
          display: flex;
          flex-direction: column;
          height: 100vh;
          background: var(--color-bg);
        }
        .pos-header {
          padding: 16px 24px;
          border-bottom: 1px solid var(--color-border);
        }
        .pos-layout {
          display: grid;
          grid-template-columns: 1fr 350px;
          gap: 0;
          flex: 1;
          min-height: 0;
        }
        .pos-products {
          display: flex;
          flex-direction: column;
          border-right: 1px solid var(--color-border);
          overflow: hidden;
        }
        .pos-search {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 12px;
          border-bottom: 1px solid var(--color-border);
        }
        .pos-search input {
          flex: 1;
          padding: 8px;
          border: 1px solid var(--color-border);
          border-radius: 4px;
        }
        .pos-categories {
          display: flex;
          gap: 6px;
          padding: 12px;
          border-bottom: 1px solid var(--color-border);
          flex-wrap: wrap;
        }
        .category-btn {
          padding: 6px 12px;
          border: 1px solid var(--color-border);
          background: transparent;
          border-radius: 4px;
          cursor: pointer;
          font-size: 0.875rem;
          transition: all 0.2s;
        }
        .category-btn:hover {
          border-color: var(--color-primary);
        }
        .category-btn.active {
          background: var(--color-primary);
          color: white;
          border-color: var(--color-primary);
        }
        .pos-products-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
          gap: 12px;
          padding: 12px;
          overflow-y: auto;
        }
        .product-card {
          display: flex;
          flex-direction: column;
          padding: 12px;
          border: 1px solid var(--color-border);
          border-radius: 6px;
          background: var(--color-hover);
          cursor: pointer;
          transition: all 0.2s;
          text-align: center;
        }
        .product-card:hover {
          border-color: var(--color-primary);
          transform: translateY(-2px);
        }
        .product-name {
          font-weight: 600;
          font-size: 0.875rem;
          margin-bottom: 4px;
        }
        .product-sku {
          font-size: 0.75rem;
          color: var(--color-text-secondary);
          margin-bottom: 6px;
        }
        .product-price {
          font-weight: 700;
          color: var(--color-primary);
          font-size: 0.875rem;
        }
        .pos-cart-section {
          display: flex;
          flex-direction: column;
          background: var(--color-hover);
          overflow: hidden;
        }
        .cart-header {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 12px;
          border-bottom: 1px solid var(--color-border);
        }
        .cart-header h2 {
          flex: 1;
          margin: 0;
          font-size: 1rem;
        }
        .cart-count {
          background: var(--color-primary);
          color: white;
          padding: 2px 8px;
          border-radius: 12px;
          font-size: 0.75rem;
          font-weight: 600;
        }
        .cart-items {
          flex: 1;
          overflow-y: auto;
          border-bottom: 1px solid var(--color-border);
        }
        .empty-cart {
          padding: 24px 12px;
          text-align: center;
          color: var(--color-text-secondary);
        }
        .cart-item {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 12px;
          border-bottom: 1px solid var(--color-border);
        }
        .item-info {
          flex: 1;
          min-width: 0;
        }
        .item-name {
          font-weight: 600;
          font-size: 0.875rem;
          overflow: hidden;
          text-overflow: ellipsis;
        }
        .item-price {
          font-size: 0.75rem;
          color: var(--color-text-secondary);
        }
        .item-qty {
          display: flex;
          gap: 4px;
          align-items: center;
        }
        .item-qty button {
          width: 24px;
          height: 24px;
          padding: 0;
          border: 1px solid var(--color-border);
          background: white;
          cursor: pointer;
          font-size: 0.875rem;
        }
        .item-qty input {
          width: 40px;
          height: 24px;
          padding: 0 4px;
          border: 1px solid var(--color-border);
          text-align: center;
          font-size: 0.75rem;
        }
        .remove-btn {
          width: 24px;
          height: 24px;
          padding: 0;
          border: 1px solid #dc3545;
          background: transparent;
          color: #dc3545;
          cursor: pointer;
          border-radius: 4px;
        }
        .cart-totals {
          padding: 12px;
          background: white;
        }
        .total-row {
          display: flex;
          justify-content: space-between;
          padding: 8px 0;
          font-size: 0.875rem;
        }
        .total-row.highlight {
          font-size: 1rem;
          border-top: 2px solid var(--color-border);
          border-bottom: 2px solid var(--color-border);
          padding: 12px 0;
          margin: 8px 0;
          font-weight: 700;
        }
        .discount-input,
        .payment-method,
        .payment-amount {
          margin: 12px 0;
        }
        .discount-input label,
        .payment-method label,
        .payment-amount label {
          display: block;
          font-size: 0.75rem;
          font-weight: 600;
          margin-bottom: 4px;
          color: var(--color-text-secondary);
        }
        .discount-input input,
        .payment-method select,
        .payment-amount input {
          width: 100%;
          padding: 8px;
          border: 1px solid var(--color-border);
          border-radius: 4px;
          font-size: 0.875rem;
        }
        .btn-full {
          width: 100%;
          margin-top: 12px;
        }
        .pos-receipt-modal {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }
        .receipt-content {
          background: white;
          padding: 32px;
          border-radius: 8px;
          text-align: center;
          box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
        }
        .receipt-content h2 {
          margin: 0 0 24px;
          font-size: 1.5rem;
          color: var(--color-primary);
        }
        .receipt-info {
          margin-bottom: 24px;
          text-align: left;
        }
        .receipt-info p {
          margin: 8px 0;
          font-size: 0.875rem;
        }
      `}</style>
    </div>
  );
}
