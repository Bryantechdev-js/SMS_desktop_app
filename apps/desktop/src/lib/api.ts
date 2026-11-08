const API = import.meta.env.VITE_API_BASE_URL || "http://127.0.0.1:8080";
let accessToken: string | null = null;
export function setAccessToken(token: string | null) { accessToken = token; }
export type Product = { id:string; sku:string; name:string; category:string; unit:string; selling_price:string; active:boolean; created_at:string };
export type ProductDetail = { 
  id:string; sku:string; name:string; description:string; category:string; product_type:string; 
  unit:string; barcode?:string; selling_price:string; purchase_price?:string; tax_rate:string; 
  supplier_id?:string; minimum_stock_threshold?:string; reorder_quantity?:string; 
  maximum_stock_threshold?:string; storage_location?:string; is_perishable:boolean; 
  image_path?:string; active:boolean; created_at:string; updated_at:string;
};
export type Balance = { product_id:string; sku:string; name:string; unit:string; quantity:string };
export type User = { id:string; username:string; display_name:string; roles:string[] };
export type StockMovement = { 
  id:string; product_id:string; kind:string; quantity_delta:string; reason:string; 
  lot_id?:string; batch_code?:string; actor_user_id?:string; reference_type?:string; 
  reference_id?:string; created_at:string;
};
export type StockStatus = {
  product_id:string; sku:string; name:string; unit:string; total_quantity:string;
  low_stock:boolean; minimum_threshold?:string;
};
export type PhysicalCount = {
  id:string; count_number:string; product_id:string; sku:string; name:string;
  counted_quantity:string; system_quantity:string; variance:string; counted_at:string; counted_by_id:string;
};
export type CheckoutLineItem = { product_id:string; quantity:string; unit_price:string; discount_amount:string; };
export type CheckoutRequest = {
  items:CheckoutLineItem[]; discount_amount:string; discount_reason?:string; customer_id?:string;
  payment_method:string; payment_amount:string; payment_reference?:string;
};
export type CheckoutResponse = {
  id:string; ticket_number:string; subtotal:string; discount_amount:string; tax_amount:string;
  total_amount:string; payment_status:string; created_at:string;
};
export type Sale = {
  id:string; ticket_number:string; cashier_id:string; status:string; subtotal:string; discount_amount:string;
  tax_amount:string; total_amount:string; currency:string; created_at:string; paid_at?:string;
};
export type SaleLine = {
  id:string; sale_id:string; product_id:string; product_name_snapshot:string; sku_snapshot:string;
  quantity:string; unit_price:string; discount_amount:string; line_total:string;
};
export type SaleDetail = {
  id:string; ticket_number:string; cashier_name:string; status:string; subtotal:string;
  discount_amount:string; tax_amount:string; total_amount:string; items:SaleLine[];
  payment_method:string; payment_status:string; created_at:string;
};

async function request<T>(path:string, init?:RequestInit):Promise<T>{
  const headers:Record<string,string>={"Content-Type":"application/json",...(init?.headers as Record<string,string>||{})};
  if(accessToken) headers.Authorization=`Bearer ${accessToken}`;
  const response=await fetch(`${API}${path}`,{...init,headers});
  if(!response.ok){const msg=await response.text();throw new Error(msg || `Request failed (${response.status})`);}
  if(response.status===204) return undefined as T;
  return response.json() as Promise<T>;
}
// Procurement types
export type Supplier = {
  id: string;
  name: string;
  contact_person: string;
  phone: string;
  email: string;
  physical_address?: string;
  payment_terms?: string;
  tax_id?: string;
  active: boolean;
};

export type PurchaseOrder = {
  id: string;
  po_number: string;
  supplier_id: string;
  expected_delivery_date?: string;
  status: string;
  subtotal_amount: string;
  created_at: string;
};

export type PurchaseReceipt = {
  id: string;
  receipt_number: string;
  purchase_order_id: string;
  receipt_date: string;
  supplier_invoice_number?: string;
  created_at: string;
};

export const api={
  health:()=>request<{status:string;service:string}>("/health"),
  login:(username:string,password:string)=>request<{access_token:string;expires_in_seconds:number;user:User}>("/api/v1/auth/login",{method:"POST",body:JSON.stringify({username,password})}),
  bootstrap:(data:{bootstrap_secret:string;username:string;display_name:string;password:string})=>request<User>("/api/v1/auth/bootstrap",{method:"POST",body:JSON.stringify(data)}),
  logout:()=>request<void>("/api/v1/auth/logout",{method:"POST"}),
  me:()=>request<User>("/api/v1/auth/me"),
  products:()=>request<Product[]>("/api/v1/products"),
  getProduct:(id:string)=>request<ProductDetail>(`/api/v1/products/${id}`),
  createProduct:(data:Omit<Product,"id"|"active"|"created_at">)=>request<Product>("/api/v1/products",{method:"POST",body:JSON.stringify(data)}),
  updateProduct:(id:string,data:Partial<ProductDetail>)=>request<ProductDetail>(`/api/v1/products/${id}`,{method:"PUT",body:JSON.stringify(data)}),
  archiveProduct:(id:string,reason:string)=>request<void>(`/api/v1/products/${id}/archive`,{method:"POST",body:JSON.stringify({reason})}),
  reactivateProduct:(id:string)=>request<void>(`/api/v1/products/${id}/reactivate`,{method:"POST"}),
  balances:()=>request<Balance[]>("/api/v1/inventory/balances"),
  movements:(productId?:string,kind?:string)=>request<StockMovement[]>(`/api/v1/inventory/movements?${new URLSearchParams({...(productId?{product_id:productId}:{}), ...(kind?{kind}:{})}).toString()}`),
  stockStatus:(category?:string,lowStockOnly?:boolean)=>request<StockStatus[]>(`/api/v1/inventory/stock-status?${new URLSearchParams({...(category?{category}:{}), ...(lowStockOnly?{low_stock_only:"true"}:{})}).toString()}`),
  createPhysicalCount:(data:{product_id:string;lot_id?:string;counted_quantity:string;notes?:string})=>request<PhysicalCount>("/api/v1/inventory/physical-counts",{method:"POST",body:JSON.stringify(data)}),
  checkout:(data:CheckoutRequest)=>request<CheckoutResponse>("/api/v1/sales/checkout",{method:"POST",body:JSON.stringify(data)}),
  listSales:(limit?:number,offset?:number)=>request<Sale[]>(`/api/v1/sales?${new URLSearchParams({...(limit?{limit:limit.toString()}:{}), ...(offset?{offset:offset.toString()}:{})}).toString()}`),
  getSale:(id:string)=>request<SaleDetail>(`/api/v1/sales/${id}`),
  // Procurement endpoints
  createSupplier:(data:{name:string;contact_person:string;phone:string;email:string;physical_address?:string;payment_terms?:string;tax_id?:string})=>request<Supplier>("/api/v1/procurement/suppliers",{method:"POST",body:JSON.stringify(data)}),
  listSuppliers:(q?:string,active?:boolean,limit?:number,offset?:number)=>request<{suppliers:Supplier[];total:number}>(`/api/v1/procurement/suppliers?${new URLSearchParams({...(q?{q}:{}), ...(active!==undefined?{active:active.toString()}:{}), ...(limit?{limit:limit.toString()}:{}), ...(offset?{offset:offset.toString()}:{})}).toString()}`),
  getSupplier:(id:string)=>request<Supplier>(`/api/v1/procurement/suppliers/${id}`),
  archiveSupplier:(id:string)=>request<{id:string;active:boolean}>(`/api/v1/procurement/suppliers/${id}/archive`,{method:"POST",body:JSON.stringify({})}),
  createPurchaseOrder:(data:{supplier_id:string;expected_delivery_date?:string;line_items:{product_id:string;quantity_ordered:string;unit_price:string;notes?:string}[]})=>request<PurchaseOrder>("/api/v1/procurement/purchase-orders",{method:"POST",body:JSON.stringify(data)}),
  listPurchaseOrders:(limit?:number,offset?:number)=>request<{purchase_orders:PurchaseOrder[];total:number}>(`/api/v1/procurement/purchase-orders?${new URLSearchParams({...(limit?{limit:limit.toString()}:{}), ...(offset?{offset:offset.toString()}:{})}).toString()}`),
  getPurchaseOrder:(id:string)=>request<PurchaseOrder>(`/api/v1/procurement/purchase-orders/${id}`),
  submitPurchaseOrder:(id:string)=>request<{id:string;po_number:string;status:string}>(`/api/v1/procurement/purchase-orders/${id}/submit`,{method:"POST",body:JSON.stringify({})}),
  cancelPurchaseOrder:(id:string,reason:string)=>request<{id:string;po_number:string;status:string}>(`/api/v1/procurement/purchase-orders/${id}/cancel`,{method:"POST",body:JSON.stringify({reason})}),
  createReceipt:(data:{purchase_order_id:string;receipt_date:string;supplier_invoice_number?:string;supplier_invoice_date?:string;line_items:{purchase_order_item_id:string;quantity_received:string;quality_status:string;discrepancy_notes?:string}[]})=>request<PurchaseReceipt>("/api/v1/procurement/receipts",{method:"POST",body:JSON.stringify(data)}),
  listReceipts:(limit?:number,offset?:number)=>request<{receipts:PurchaseReceipt[];total:number}>(`/api/v1/procurement/receipts?${new URLSearchParams({...(limit?{limit:limit.toString()}:{}), ...(offset?{offset:offset.toString()}:{})}).toString()}`),
  getReceipt:(id:string)=>request<PurchaseReceipt>(`/api/v1/procurement/receipts/${id}`)
};
