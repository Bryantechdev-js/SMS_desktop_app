import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { api, PurchaseReceipt, PurchaseOrder } from '../../lib/api';

interface ReceiptLineItem {
  purchase_order_item_id: string;
  quantity_received: string;
  quality_status: string;
  discrepancy_notes?: string;
  temp_id?: string;
}

interface CreateReceiptRequest {
  purchase_order_id: string;
  receipt_date: string;
  supplier_invoice_number?: string;
  supplier_invoice_date?: string;
  line_items: ReceiptLineItem[];
}

export default function ReceiptTab() {
  const { t } = useTranslation();
  const [receipts, setReceipts] = useState<PurchaseReceipt[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [dateFrom, setDateFrom] = useState('');
  const [dateTo, setDateTo] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [pos, setPOs] = useState<PurchaseOrder[]>([]);
  const [formData, setFormData] = useState<CreateReceiptRequest>({
    purchase_order_id: '',
    receipt_date: new Date().toISOString().split('T')[0],
    supplier_invoice_number: '',
    supplier_invoice_date: '',
    line_items: [],
  });

  useEffect(() => {
    loadReceipts();
  }, []);

  useEffect(() => {
    if (showForm) {
      loadPOs();
    }
  }, [showForm]);

  const loadReceipts = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await api.listReceipts(50, 0);
      setReceipts(response.receipts || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load receipts');
    } finally {
      setLoading(false);
    }
  };

  const loadPOs = async () => {
    try {
      const response = await api.listPurchaseOrders(100, 0);
      // Filter for submitted POs only
      const submittedPOs = (response.purchase_orders || []).filter(
        (po) => po.status === 'submitted'
      );
      setPOs(submittedPOs);
    } catch (err) {
      console.error('Failed to load POs:', err);
    }
  };

  const handleSelectPO = async (poId: string) => {
    setFormData({
      ...formData,
      purchase_order_id: poId,
      line_items: [],
    });

    // In a real app, we'd fetch the PO details to get line items
    // For now, use placeholder line items based on PO
    // This would require an API endpoint to get PO details with items
  };

  const handleAddLineItem = () => {
    const newItem: ReceiptLineItem = {
      purchase_order_item_id: '',
      quantity_received: '',
      quality_status: 'accepted',
      discrepancy_notes: '',
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

  const handleUpdateLineItem = (tempId: string | undefined, updates: Partial<ReceiptLineItem>) => {
    setFormData({
      ...formData,
      line_items: formData.line_items.map((item) =>
        item.temp_id === tempId ? { ...item, ...updates } : item
      ),
    });
  };

  const handleCreateReceipt = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!formData.purchase_order_id) {
      setError(t('procurement.selectPO'));
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
      const response = await api.createReceipt(sanitizedData);
      setReceipts([response, ...receipts]);
      setFormData({
        purchase_order_id: '',
        receipt_date: new Date().toISOString().split('T')[0],
        supplier_invoice_number: '',
        supplier_invoice_date: '',
        line_items: [],
      });
      setShowForm(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create receipt');
    }
  };

  const filteredReceipts = receipts.filter((r) =>
    r.receipt_number.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <div className="tab-panel">
      <div className="panel-header">
        <div className="search-box">
          <input
            type="text"
            placeholder={t('procurement.receiptNumber')}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="search-input"
          />
        </div>
        <div className="date-filters">
          <input
            type="date"
            value={dateFrom}
            onChange={(e) => setDateFrom(e.target.value)}
            className="form-input"
            placeholder={t('common.date')}
          />
          <span> - </span>
          <input
            type="date"
            value={dateTo}
            onChange={(e) => setDateTo(e.target.value)}
            className="form-input"
            placeholder={t('common.date')}
          />
          <button className="btn btn-secondary" onClick={() => loadReceipts()}>
            {t('common.refresh')}
          </button>
        </div>
        <button className="btn btn-primary" onClick={() => setShowForm(!showForm)}>
          {showForm ? t('common.cancel') : t('procurement.createReceipt')}
        </button>
      </div>

      {error && <div className="alert alert-error">{error}</div>}

      {showForm && (
        <form className="form-container" onSubmit={handleCreateReceipt}>
          <div className="form-row">
            <div className="form-group">
              <label>{t('procurement.purchaseOrder')} *</label>
              <select
                required
                value={formData.purchase_order_id}
                onChange={(e) => handleSelectPO(e.target.value)}
                className="form-select"
              >
                <option value="">{t('common.select')}</option>
                {pos.map((po) => (
                  <option key={po.id} value={po.id}>
                    {po.po_number}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-group">
              <label>{t('procurement.receiptDate')} *</label>
              <input
                type="date"
                required
                value={formData.receipt_date}
                onChange={(e) => setFormData({ ...formData, receipt_date: e.target.value })}
                className="form-input"
              />
            </div>
          </div>

          <div className="form-row">
            <div className="form-group">
              <label>{t('procurement.supplierInvoiceNumber')}</label>
              <input
                type="text"
                value={formData.supplier_invoice_number || ''}
                onChange={(e) =>
                  setFormData({ ...formData, supplier_invoice_number: e.target.value })
                }
                className="form-input"
              />
            </div>
            <div className="form-group">
              <label>{t('procurement.supplierInvoiceDate')}</label>
              <input
                type="date"
                value={formData.supplier_invoice_date || ''}
                onChange={(e) =>
                  setFormData({ ...formData, supplier_invoice_date: e.target.value })
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
                  <th>{t('procurement.quantityReceived')}</th>
                  <th>{t('procurement.qualityStatus')}</th>
                  <th>{t('procurement.discrepancyNotes')}</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {formData.line_items.map((item) => (
                  <tr key={item.temp_id}>
                    <td>
                      <input
                        type="text"
                        required
                        value={item.purchase_order_item_id}
                        onChange={(e) =>
                          handleUpdateLineItem(item.temp_id, {
                            purchase_order_item_id: e.target.value,
                          })
                        }
                        className="form-input"
                        placeholder={t('procurement.poItemId')}
                      />
                    </td>
                    <td>
                      <input
                        type="number"
                        required
                        min="0.01"
                        step="0.01"
                        value={item.quantity_received}
                        onChange={(e) =>
                          handleUpdateLineItem(item.temp_id, {
                            quantity_received: e.target.value,
                          })
                        }
                        className="form-input"
                        placeholder="0"
                      />
                    </td>
                    <td>
                      <select
                        required
                        value={item.quality_status}
                        onChange={(e) =>
                          handleUpdateLineItem(item.temp_id, {
                            quality_status: e.target.value,
                          })
                        }
                        className="form-select"
                      >
                        <option value="accepted">{t('procurement.accepted')}</option>
                        <option value="rejected">{t('procurement.rejected')}</option>
                        <option value="hold_for_inspection">
                          {t('procurement.holdForInspection')}
                        </option>
                      </select>
                    </td>
                    <td>
                      <input
                        type="text"
                        value={item.discrepancy_notes || ''}
                        onChange={(e) =>
                          handleUpdateLineItem(item.temp_id, {
                            discrepancy_notes: e.target.value,
                          })
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
      ) : filteredReceipts.length === 0 ? (
        <div className="empty-state">{t('common.empty')}</div>
      ) : (
        <div className="list-container">
          {filteredReceipts.map((receipt) => (
            <div key={receipt.id} className="list-item">
              <div className="list-item-header">
                <h3>{receipt.receipt_number}</h3>
              </div>
              <div className="list-item-details">
                <p>
                  <strong>{t('procurement.receiptDate')}:</strong> {receipt.receipt_date}
                </p>
                {receipt.supplier_invoice_number && (
                  <p>
                    <strong>{t('procurement.supplierInvoiceNumber')}:</strong>{' '}
                    {receipt.supplier_invoice_number}
                  </p>
                )}
                <p>
                  <strong>{t('procurement.orderDate')}:</strong>{' '}
                  {new Date(receipt.created_at).toLocaleDateString()}
                </p>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
