import { useEffect, useMemo, useState, type FormEvent, type ReactNode } from "react";
import { Activity, Bell, Boxes, ChevronDown, CircleHelp, ClipboardList, Factory, Globe2, LayoutDashboard, PackagePlus, Search, Settings, ShoppingCart, Truck, Wallet, Wheat } from "lucide-react";
import { useTranslation } from "react-i18next";
import i18n from "./i18n/config";
import { api, setAccessToken, type Balance, type Product } from "./lib/api";
import { InventoryPage } from "./pages/Inventory";
import { POSPage } from "./pages/POS";
import { SettingsPage } from "./pages/Settings";

type Page = "Overview" | "POS" | "Inventory" | "Production" | "Procurement" | "Cash" | "Settings";

const nav: { name: Page; icon: typeof LayoutDashboard; group: string }[] = [
  { name: "Overview", icon: LayoutDashboard, group: "WORKSPACE" },
  { name: "POS", icon: ShoppingCart, group: "WORKSPACE" },
  { name: "Inventory", icon: Boxes, group: "WORKSPACE" },
  { name: "Production", icon: Factory, group: "WORKSPACE" },
  { name: "Procurement", icon: Truck, group: "WORKSPACE" },
  { name: "Cash", icon: Wallet, group: "WORKSPACE" },
  { name: "Settings", icon: Settings, group: "SYSTEM" }
];

function Workspace() {
  const { t } = useTranslation();
  const [page, setPage] = useState<Page>("Overview");
  const [currentUser, setCurrentUser] = useState("Administrator");
  const [products, setProducts] = useState<Product[]>([]);
  const [balances, setBalances] = useState<Balance[]>([]);
  const [online, setOnline] = useState(false);
  const [error, setError] = useState("");
  const [query, setQuery] = useState("");
  const [showAdd, setShowAdd] = useState(false);
  const [name, setName] = useState("");
  const [sku, setSku] = useState("");
  const [price, setPrice] = useState("0");
  const [category, setCategory] = useState("Bakery");
  const [unit, setUnit] = useState("piece");
  const [busy, setBusy] = useState(false);

  async function signOut() {
    try {
      await api.logout();
    } catch {
      // Logout attempt made
    }
    setAccessToken(null);
    window.location.reload();
  }

  async function refresh() {
    try {
      await api.health();
      const [me, p, b] = await Promise.all([api.me(), api.products(), api.balances()]);
      setCurrentUser(me.display_name);
      setProducts(p);
      setBalances(b);
      setOnline(true);
      setError("");
    } catch (e) {
      setOnline(false);
      setError(e instanceof Error ? e.message : "API unavailable");
    }
  }

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => void refresh(), 30000);
    return () => window.clearInterval(id);
  }, []);

  const filtered = useMemo(
    () => products.filter((p) => `${p.name} ${p.sku} ${p.category}`.toLowerCase().includes(query.toLowerCase())),
    [products, query]
  );

  const totalUnits = balances.reduce((sum, b) => sum + Number(b.quantity), 0);
  const lowStock = balances.filter((b) => Number(b.quantity) <= 5).length;

  async function addProduct(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      await api.createProduct({
        name,
        sku,
        category,
        unit,
        selling_price: price
      });
      setName("");
      setSku("");
      setPrice("0");
      setShowAdd(false);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not create product");
    } finally {
      setBusy(false);
    }
  }

  const todayDate = new Date().toLocaleDateString(i18n.language === "fr" ? "fr-FR" : "en-US", {
    weekday: "long",
    day: "2-digit",
    month: "long",
    year: "numeric"
  }).toUpperCase();

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">
            <Wheat size={22} />
          </div>
          <div>
            <strong>COMPLEXE <span>SMS</span></strong>
            <small>{t("dashboard.welcome")} BAKERY OPERATIONS</small>
          </div>
        </div>

        <div className="branch">
          <div className="branch-dot" />
          <div>
            <b>Main Bakery</b>
            <small>Douala · Cameroon</small>
          </div>
          <ChevronDown size={15} />
        </div>

        {(["WORKSPACE", "SYSTEM"] as const).map((group) => (
          <div className="nav-group" key={group}>
            <p>{group}</p>
            {nav
              .filter((n) => n.group === group)
              .map((item) => (
                <button
                  key={item.name}
                  onClick={() => setPage(item.name)}
                  className={`nav-item ${page === item.name ? "active" : ""}`}
                >
                  <item.icon size={18} />
                  <span>{t(`navigation.${item.name.toLowerCase() as keyof typeof t}`)}</span>
                  {item.name === "POS" && <kbd>F2</kbd>}
                </button>
              ))}
          </div>
        ))}

        <div className="sidebar-bottom">
          <div className="help-card">
            <div className="help-icon">
              <CircleHelp size={18} />
            </div>
            <b>{t("common.help")}</b>
            <p>Explore quick guides and learn how to use your workspace.</p>
            <button onClick={() => setPage("Settings")}>
              {t("common.help")} centre <span>↗</span>
            </button>
          </div>

          <div className="profile">
            <div className="avatar">
              {currentUser
                .split(" ")
                .map((x) => x[0])
                .slice(0, 2)
                .join("")
                .toUpperCase()}
            </div>
            <div>
              <b>{currentUser}</b>
              <small>System workspace</small>
            </div>
            <button aria-label="Profile menu">
              <ChevronDown size={16} />
            </button>
          </div>
        </div>
      </aside>

      <main className="main">
        <header className="topbar">
          <div className="breadcrumbs">
            {t("navigation.overview")} <span>/</span> <b>{page !== "Overview" ? page : ""}</b>
          </div>
          <div className="top-actions">
            <div className={`connection ${online ? "connected" : "disconnected"}`}>
              <i />
              {online ? "API connected" : "API disconnected"}
            </div>
            <button className="icon-btn" aria-label="Language" onClick={() => setPage("Settings")}>
              <Globe2 size={18} />
              <span>{i18n.language.toUpperCase()}</span>
            </button>
            <button className="icon-btn" aria-label="Notifications">
              <Bell size={18} />
              <em />
            </button>
            <button className="top-avatar" title="Sign out" onClick={() => void signOut()}>
              {currentUser
                .split(" ")
                .map((x) => x[0])
                .slice(0, 2)
                .join("")
                .toUpperCase()}
            </button>
          </div>
        </header>

        <section className="content">
          <div className="page-heading">
            <div>
              <div className="eyebrow">{todayDate}</div>
              <h1>{page === "Overview" ? `${t("common.welcome")}, ${currentUser}` : ""}{page !== "Overview" ? page : ""}</h1>
              <p>{page === "Overview" ? t("dashboard.operationalWorkflow") : description(page, t)}</p>
            </div>
            <div className="heading-actions">
              <button className="btn btn-secondary" onClick={() => void refresh()}>
                <Activity size={16} /> {t("common.refresh")}
              </button>
              {page === "Inventory" && (
                <button className="btn btn-primary" onClick={() => setShowAdd(true)}>
                  <PackagePlus size={16} /> {t("inventory.addProduct")}
                </button>
              )}
            </div>
          </div>

          {error && (
            <div className="notice">
              <strong>{t("errors.backendConnectionIssue")}</strong>
              <span>{error}. Start PostgreSQL and the Rust API, then refresh.</span>
            </div>
          )}

          {page === "Overview" && (
            <>
              <div className="stats-grid">
                <Stat
                  label={t("dashboard.productsInCatalog")}
                  value={String(products.length)}
                  note={t("dashboard.acrossTrackedProducts")}
                  icon={<Boxes />}
                />
                <Stat
                  label={t("dashboard.stockUnits")}
                  value={totalUnits.toLocaleString()}
                  note={t("dashboard.acrossTrackedProducts")}
                  icon={<PackagePlus />}
                />
                <Stat
                  label={t("dashboard.lowStockItems")}
                  value={String(lowStock)}
                  note={t("dashboard.atOrBelow")}
                  icon={<Activity />}
                  warn={lowStock > 0}
                />
                <Stat
                  label={t("dashboard.apiStatus")}
                  value={online ? t("dashboard.online") : t("dashboard.offline")}
                  note={t("dashboard.localDevelopmentConnection")}
                  icon={<Globe2 />}
                />
              </div>

              <div className="dashboard-grid">
                <div className="panel wide">
                  <div className="panel-head">
                    <div>
                      <h2>{t("dashboard.inventorySnapshot")}</h2>
                      <p>{t("dashboard.currentBalances")}</p>
                    </div>
                    <button className="text-btn" onClick={() => setPage("Inventory")}>
                      {t("dashboard.viewInventory")} ↗
                    </button>
                  </div>
                  <InventoryTable rows={balances.slice(0, 6)} />
                </div>

                <div className="panel">
                  <div className="panel-head">
                    <div>
                      <h2>{t("dashboard.quickActions")}</h2>
                      <p>{t("dashboard.commonWorkspaceTasks")}</p>
                    </div>
                  </div>
                  <div className="quick-actions">
                    <Quick
                      icon={<ShoppingCart />}
                      title={t("pos.title")}
                      desc="Start a customer transaction"
                      onClick={() => setPage("POS")}
                    />
                    <Quick
                      icon={<PackagePlus />}
                      title={t("inventory.addProduct")}
                      desc="Create a catalog item"
                      onClick={() => {
                        setPage("Inventory");
                        setShowAdd(true);
                      }}
                    />
                    <Quick
                      icon={<Truck />}
                      title="Record procurement"
                      desc="Manage supplier deliveries"
                      onClick={() => setPage("Procurement")}
                    />
                    <Quick
                      icon={<ClipboardList />}
                      title="Plan production"
                      desc="Prepare a production batch"
                      onClick={() => setPage("Production")}
                    />
                  </div>
                </div>
              </div>

              <div className="panel lower-panel">
                <div className="panel-head">
                  <div>
                    <h2>{t("dashboard.operationalWorkflow")}</h2>
                    <p>{t("dashboard.coreBakeryProcess")}</p>
                  </div>
                  <span className="tag">{t("dashboard.systemOverview")}</span>
                </div>
                <div className="workflow">
                  {[
                    { n: "01", t: "Procurement", d: "Receive ingredients", i: <Truck /> },
                    { n: "02", t: "Inventory", d: "Track raw materials", i: <Boxes /> },
                    { n: "03", t: "Production", d: "Make finished goods", i: <Factory /> },
                    { n: "04", t: "Point of sale", d: "Sell and issue receipts", i: <ShoppingCart /> },
                    { n: "05", t: "Cash", d: "Reconcile payments", i: <Wallet /> }
                  ].map((x, i) => (
                    <div className="workflow-step" key={x.n}>
                      <div className="step-icon">{x.i}</div>
                      <small>{x.n}</small>
                      <b>{x.t}</b>
                      <span>{x.d}</span>
                      {i !== 4 && <div className="step-line" />}
                    </div>
                  ))}
                </div>
              </div>
            </>
          )}

          {page === "Inventory" && <InventoryPage onError={setError} />}
          {page === "POS" && <POSPage onError={setError} />}
          {page === "Settings" && <SettingsPage />}

          <footer>
            © 2026 Complexe SMS <span>Powered by <b>Sinior.dev</b></span>
            <span>Development build · v0.1.0</span>
          </footer>
        </section>
      </main>

      {showAdd && (
        <div
          className="modal-backdrop"
          onMouseDown={(e) => {
            if (e.target === e.currentTarget) setShowAdd(false);
          }}
        >
          <form className="modal" onSubmit={addProduct}>
            <div className="modal-head">
              <div>
                <h2>{t("inventory.addProduct")}</h2>
                <p>Create a product record for inventory tracking.</p>
              </div>
              <button type="button" className="close" onClick={() => setShowAdd(false)}>
                ×
              </button>
            </div>
            <label>
              {t("inventory.productName")}
              <input
                required
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Croissant"
              />
            </label>
            <label>
              {t("inventory.sku")}
              <input
                required
                value={sku}
                onChange={(e) => setSku(e.target.value)}
                placeholder="e.g. BAK-001"
              />
            </label>
            <div className="form-row">
              <label>
                {t("inventory.category")}
                <input value={category} onChange={(e) => setCategory(e.target.value)} />
              </label>
              <label>
                {t("inventory.unit")}
                <input value={unit} onChange={(e) => setUnit(e.target.value)} />
              </label>
            </div>
            <label>
              {t("inventory.sellingPrice")}
              <input
                type="number"
                min="0"
                step="1"
                required
                value={price}
                onChange={(e) => setPrice(e.target.value)}
              />
            </label>
            <div className="modal-note">
              This creates a catalog record only. It does not add stock. Use a verified stock receipt or opening balance workflow when implemented.
            </div>
            <div className="modal-actions">
              <button type="button" className="btn btn-secondary" onClick={() => setShowAdd(false)}>
                {t("common.cancel")}
              </button>
              <button className="btn btn-primary" disabled={busy}>
                {busy ? `${t("common.loading")}` : `${t("inventory.addProduct")}`}
              </button>
            </div>
          </form>
        </div>
      )}
    </div>
  );
}

function description(p: Page, t: any): string {
  return ({
    POS: t("pos.title"),
    Inventory: t("inventory.title"),
    Production: "Plan batches, recipes, consumption and finished output.",
    Procurement: "Manage suppliers, purchase orders and goods receiving.",
    Cash: "Manage cashier sessions, expenses and reconciliation.",
    Settings: t("settings.title"),
    Overview: t("dashboard.operationalWorkflow")
  })[p];
}

function Stat({
  label,
  value,
  note,
  icon,
  warn = false
}: {
  label: string;
  value: string;
  note: string;
  icon: ReactNode;
  warn?: boolean;
}) {
  return (
    <div className="stat-card">
      <div className="stat-top">
        <span>{label}</span>
        <div className={`stat-icon ${warn ? "warn" : ""}`}>{icon}</div>
      </div>
      <strong>{value}</strong>
      <small>{note}</small>
    </div>
  );
}

function InventoryTable({ rows }: { rows: Balance[] }) {
  const { t } = useTranslation();
  
  return (
    <div className="table-wrap">
      <table>
        <thead>
          <tr>
            <th>{t("inventory.productName")}</th>
            <th>{t("inventory.sku")}</th>
            <th>{t("inventory.unit")}</th>
            <th>{t("inventory.onHand")}</th>
            <th>{t("common.status")}</th>
          </tr>
        </thead>
        <tbody>
          {rows.length ? (
            rows.map((r) => (
              <tr key={r.product_id}>
                <td>
                  <b>{r.name}</b>
                </td>
                <td className="muted">{r.sku}</td>
                <td>{r.unit}</td>
                <td className="quantity">{Number(r.quantity).toLocaleString()}</td>
                <td>
                  <span className={`status ${Number(r.quantity) <= 5 ? "low" : "healthy"}`}>
                    {Number(r.quantity) <= 5 ? t("inventory.lowStockAlert") : t("inventory.inStock")}
                  </span>
                </td>
              </tr>
            ))
          ) : (
            <tr>
              <td colSpan={5} className="empty">
                {t("common.empty")}
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

function Quick({
  icon,
  title,
  desc,
  onClick
}: {
  icon: ReactNode;
  title: string;
  desc: string;
  onClick: () => void;
}) {
  return (
    <button className="quick" onClick={onClick}>
      <div>{icon}</div>
      <span>
        <b>{title}</b>
        <small>{desc}</small>
      </span>
      <span className="quick-arrow">↗</span>
    </button>
  );
}

function AuthScreen({ onAuthenticated }: { onAuthenticated: () => void }) {
  const { t } = useTranslation();
  const [mode, setMode] = useState<"login" | "setup">("login");
  const [username, setUsername] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [secret, setSecret] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");

  async function submit(e: FormEvent) {
    e.preventDefault();
    setBusy(true);
    setError("");
    try {
      if (mode === "setup") {
        await api.bootstrap({
          bootstrap_secret: secret,
          username,
          display_name: displayName,
          password
        });
      }
      const result = await api.login(username, password);
      setAccessToken(result.access_token);
      onAuthenticated();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Authentication failed");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="auth-shell">
      <div className="auth-card">
        <div className="auth-brand">
          <div className="brand-mark">
            <Wheat size={24} />
          </div>
          <div>
            <strong>COMPLEXE <span>SMS</span></strong>
            <small>BAKERY OPERATIONS</small>
          </div>
        </div>

        <div className="auth-intro">
          <div className="eyebrow">SECURE WORKSPACE</div>
          <h1>
            {mode === "login" ? t("auth.welcome") : t("auth.setupWorkspace")}
          </h1>
          <p>
            {mode === "login"
              ? t("auth.welcome")
              : "Create the first owner account. The setup secret is provided by your system administrator."}
          </p>
        </div>

        <form onSubmit={submit} className="auth-form">
          {mode === "setup" && (
            <>
              <label>
                {t("auth.setupSecret")}
                <input
                  type="password"
                  required
                  minLength={24}
                  value={secret}
                  onChange={(e) => setSecret(e.target.value)}
                  autoComplete="off"
                />
              </label>
              <label>
                {t("auth.displayName")}
                <input
                  required
                  maxLength={120}
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  placeholder="Your full name"
                />
              </label>
            </>
          )}

          <label>
            {t("auth.username")}
            <input
              required
              minLength={3}
              maxLength={64}
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              autoComplete="username"
              placeholder="e.g. manager"
            />
          </label>

          <label>
            {t("auth.password")}
            <input
              type="password"
              required
              minLength={12}
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              autoComplete={mode === "login" ? "current-password" : "new-password"}
              placeholder="At least 12 characters"
            />
          </label>

          {error && <div className="notice auth-error">{error}</div>}

          <button className="btn btn-primary auth-submit" disabled={busy}>
            {busy ? t("common.loading") : mode === "login" ? t("auth.signIn") : t("auth.createAccount")}
          </button>
        </form>

        <button
          className="auth-switch"
          onClick={() => {
            setMode(mode === "login" ? "setup" : "login");
            setError("");
          }}
        >
          {mode === "login"
            ? "First-time setup? Create the owner account"
            : "Already configured? Return to sign in"}
        </button>

        <div className="auth-foot">Protected by individual staff accounts · Sessions expire automatically</div>
      </div>

      <div className="auth-side">
        <div className="auth-side-content">
          <Wheat size={38} />
          <h2>Every batch. Every sale. In control.</h2>
          <p>A clear, connected workspace for the people who keep your bakery moving.</p>
          <div className="auth-benefits">
            <span>Secure staff access</span>
            <span>Traceable stock movements</span>
            <span>One operational source of truth</span>
          </div>
        </div>
        <small>Powered by Sinior.dev · Douala, Cameroon</small>
      </div>
    </div>
  );
}

export default function App() {
  const [authenticated, setAuthenticated] = useState(false);

  return authenticated ? (
    <Workspace />
  ) : (
    <AuthScreen onAuthenticated={() => setAuthenticated(true)} />
  );
}
