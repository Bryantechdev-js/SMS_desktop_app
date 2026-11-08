import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { api, PurchaseOrder, Supplier, Product } from '../../lib/api';

interface POLineItem {
  product_id: string;
  quantity_ordered: string;
  unit_price: string;
  notes?: string;
  temp_id?: string;
}

interface CreatePORequest {
  supplier_id: string;
  expected_delivery_date?: string;
  line_items: POLineItem[];
}

export default function PurchaseOrderTab() {
  const { t } = useTranslation();
  const [pos, setPOs] = useState<PurchaseOrder[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('');
  const [showForm, setShowForm] = useState(false);
  const [suppliers, setSuppliers] = useState<Supplier[]>([]);
  const [products, setProducts] = useState<Product[]>([]);
  const [formData, setFormData] = useState<CreatePORequest>({
    supplier_id: '',
    expected_delivery_date: '',
    line_items: [],
  });

  useEffect(() => {
    loadPurchaseOrders();
  }, [statusFilter]);

  useEffect(() => {
    if (showForm) {
      loadSuppliers();
      loadProducts();
    }
  }, [showForm]);

  const loadPurchaseOrders = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await api.listPurchaseOrders(50, 0);
      setPOs(response.purchase_orders || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load purchase orders');
    } finally {
      setLoading(false);
    }
  };

  const loadSuppliers = async () => {
    try {
      const response = await api.listSuppliers(undefined, true, 100, 0);
      setSuppliers(response.suppliers || []);
    } catch (err) {
      console.error('Failed to load suppliers:', err);
    }
  };

  const loadProducts = async () => {
    try {
      const products = await api.products();
      setProducts(products || []);
    } catch (err) {
      console.error('Failed to load products:', err);
    }
  };

  const handleAddLineItem = () => {
    const newItem: POLineItem = {
      product_id: '',
      quantity_ordered: '',
      unit_price: '',
      notes: '',
      temp_id: Math.random().toString(),
    };
    setFormData({
      ...formData,
      line_items: [...formData.line_items, newItem],
    });
  };

  const handleRemoveLineItem = (tempId?: string) => {
    setFormData({
      ...formData,
      line_items: formData.line_items.filter((item) => item.temp_id !== tempId),
    });
  };

  const handleUpdateLineItem = (tempId: string | undefined, updates: Partial<POLineItem>) => {
    setFormData({
      ...formData,
      line_items: formData.line_items.map((item) =>
        item.temp_id === tempId ? { ...item, ...updates } : item
      ),
    });
  };

  const handleCreatePO = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!formData.supplier_id) {
      setError(t('procurement.selectSupplier'));
      return;
    }

    if (formData.line_items.length === 0) {
      setError(t('procurement.addLineItems'));
      return;
    }

    try {
      // Remove temp_id from line items before sending
      const sanitizedData = {
        ...formData,
        line_items: formData.line_items.map(({ temp_id, ...item }) => item),
      };
      const response = await api.createPurchaseOrder(sanitizedData);
      setPOs([response, ...pos]);
      setFormData({
        supplier_id: '',
        expected_delivery_date: '',
        line_items: [],
      });
      setShowForm(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create purchase order');
    }
  };

  const handleSubmitPO = async (poId: string) => {
    if (!window.confirm(t('procurement.confirmSubmit'))) return;

    try {
      const response = await api.submitPurchaseOrder(poId);
      setPOs(
        pos.map((po) => (po.id === poId ? { ...po, status: response.status } : po))
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to submit PO');
    }
  };

  const handleCancelPO = async (poId: string) => {
    const reason = window.prompt(t('procurement.cancelReason'));
    if (!reason) return;

    try {
      const response = await api.cancelPurchaseOrder(poId, reason);
      setPOs(
        pos.map((po) => (po.id === poId ? { ...po, status: response.status } : po))
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to cancel PO');
    }
  };

  const getStatusBadgeClass = (status: string) => {
    switch (status) {
      case 'draft':
        return 'badge-secondary';
      case 'submitted':
        return 'badge-info';
      case 'partial':
        return 'badge-warning';
      case 'complete':
        return 'badge-success';
      case 'cancelled':
        return 'badge-danger';
      default:
        return 'badge-secondary';
    }
  };

  const filteredPOs = pos.filter((po) => po.po_number.toLowerCase().includes(searchQuery.toLowerCase()));

  return (
    <div className="tab-panel">
      <div className="panel-header">
        <div className="search-box">
          <input
            type="text"
            placeholder={t('procurement.poNumber')}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
        </div>
        <select
          value={statusFilter}
          onChange={(e) => setStatusFilter(e.target.value)}
          className="form-select"
        >
          <option value="">{t('common.status')} - All</option>
          <option value="draft">{t('procurement.draft')}</option>
          <option value="submitted">{t('procurement.submitted')}</option>
          <option value="partial">{t('procurement.partial')}</option>
          <option value="complete">{t('procurement.complete')}</option>
          <option value="cancelled">{t('procurement.cancelled')}</option>
        </select>
        <button className="btn btn-primary" onClick={() => setShowForm(!showForm)}>
          {showForm ? t('common.cancel') : t('procurement.createPO')}
        </button>
      </div>

      {error && <div className="alert alert-error">{error}</div>}

      {showForm && (
        <form className="form-container" onSubmit={handleCreatePO}>
          <div className="form-row">
            <div className="form-group">
              <label>{t('procurement.supplier')} *</label>
              <select
                required
                value={formData.supplier_id}
                onChange={(e) => setFormData({ ...formData, supplier_id: e.target.value })}
                className="form-select"
              >
                <option value="">{t('common.select')}</option>
                {suppliers.map((s) => (
                  <option key={s.id} value={s.id}>
                    {s.name}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-group">
              <label>{t('procurement.expectedDelivery')}</label>
              <input
                type="date"
                value={formData.expected_delivery_date || ''}
                onChange={(e) =>
                  setFormData({ ...formData, expected_delivery_date: e.target.value })
                }
                className="form-input"
              />
            </div>
          </div>

          <div className="form-group">
            <label>{t('procurement.lineItems')} *</label>
            <table className="line-items-table">
              <thead>
                <tr>
                  <th>{t('procurement.product')}</th>
                  <th>{t('procurement.quantity')}</th>
                  <th>{t('procurement.unitPrice')}</th>
                  <th>{t('procurement.notes')}</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {formData.line_items.map((item) => (
                  <tr key={item.temp_id}>
                    <td>
                      <select
                        required
                        value={item.product_id}
                        onChange={(e) =>
                          handleUpdateLineItem(item.temp_id, { product_id: e.target.value })
                        }
                        className="form-select"
                      >
                        <option value="">{t('common.select')}</option>
                        {products.map((p) => (
                          <option key={p.id} value={p.id}>
                            {p.name} ({p.sku})
                          </option>
                        ))}
                      </select>
                    </td>
                    <td>
                      <input
                        type="number"
                        required
                        min="0.01"
                        step="0.01"
                        value={item.quantity_ordered}
                        onChange={(e) =>
                          handleUpdateLineItem(item.temp_id, {
                            quantity_ordered: e.target.value,
                          })
                        }
                        className="form-input"
                        placeholder="0"
                      />
                    </td>
                    <td>
                      <input
                        type="number"
                        required
                        min="0.01"
                        step="0.01"
                        value={item.unit_price}
                        onChange={(e) =>
                          handleUpdateLineItem(item.temp_id, { unit_price: e.target.value })
                        }
                        className="form-input"
                        placeholder="0.00"
                      />
                    </td>
                    <td>
                      <input
                        type="text"
                        value={item.notes || ''}
                        onChange={(e) =>
                          handleUpdateLineItem(item.temp_id, { notes: e.target.value })
                        }
                        className="form-input"
                        placeholder={t('common.optional')}
                      />
                    </td>
                    <td>
                      <button
                        type="button"
                        className="btn btn-sm btn-danger"
                        onClick={() => handleRemoveLineItem(item.temp_id)}
                      >
                        ✕
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            <button type="button" className="btn btn-secondary btn-sm" onClick={handleAddLineItem}>
              {t('procurement.addLineItem')}
            </button>
          </div>

          <div className="form-actions">
            <button type="submit" className="btn btn-primary">
              {t('common.create')}
            </button>
            <button
              type="button"
              className="btn btn-secondary"
              onClick={() => setShowForm(false)}
            >
              {t('common.cancel')}
            </button>
          </div>
        </form>
      )}

      {error && <div className="alert alert-error">{error}</div>}

      {loading ? (
        <div className="loading">{t('common.loading')}</div>
      ) : filteredPOs.length === 0 ? (
        <div className="empty-state">{t('common.empty')}</div>
      ) : (
        <div className="list-container">
          {filteredPOs.map((po) => (
            <div key={po.id} className="list-item">
              <div className="list-item-header">
                <h3>{po.po_number}</h3>
                <div className="list-item-actions">
                  <span className={`badge ${getStatusBadgeClass(po.status)}`}>
                    {t(`procurement.${po.status}`)}
                  </span>
                  {po.status === 'draft' && (
                    <button
                      className="btn btn-sm btn-primary"
                      onClick={() => handleSubmitPO(po.id)}
                    >
                      {t('procurement.submit')}
                    </button>
                  )}
                  {po.status === 'submitted' && (
                    <button
                      className="btn btn-sm btn-danger"
                      onClick={() => handleCancelPO(po.id)}
                    >
                      {t('common.cancel')}
                    </button>
                  )}
                </div>
              </div>
              <div className="list-item-details">
                <p>
                  <strong>{t('procurement.subtotalAmount')}:</strong> {po.subtotal_amount} XAF
                </p>
                {po.expected_delivery_date && (
                  <p>
                    <strong>{t('procurement.expectedDelivery')}:</strong> {po.expected_delivery_date}
                  </p>
                )}
                <p>
                  <strong>{t('procurement.orderDate')}:</strong>{' '}
                  {new Date(po.created_at).toLocaleDateString()}
                </p>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
