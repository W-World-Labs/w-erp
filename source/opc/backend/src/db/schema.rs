/// SQLite schema initialization - Multi-tenant Architecture
pub const INIT_SQL: &str = r#"
-- Users (synced from W-ID SSO)
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    name TEXT,
    avatar_url TEXT,
    github_login TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Companies (租户)
CREATE TABLE IF NOT EXISTS companies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    legal_name TEXT,
    tax_id TEXT,
    registration_number TEXT,
    address TEXT,
    city TEXT,
    province TEXT,
    postal_code TEXT,
    phone TEXT,
    email TEXT,
    website TEXT,
    industry TEXT,
    founded_date TEXT,
    description TEXT,
    logo_url TEXT,
    legal_representative TEXT,
    unified_social_credit_code TEXT,
    registered_capital REAL,
    country TEXT DEFAULT 'CN',
    timezone TEXT DEFAULT 'Asia/Shanghai',
    locale TEXT DEFAULT 'zh',
    base_currency TEXT DEFAULT 'CNY',
    date_format TEXT DEFAULT 'YYYY-MM-DD',
    is_demo INTEGER DEFAULT 0,
    is_deleted INTEGER DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Company Members (用户-公司多对多关系)
CREATE TABLE IF NOT EXISTS company_members (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    company_id TEXT NOT NULL,
    role TEXT DEFAULT 'member',
    -- 角色: owner, admin, member, accountant, auditor, tax_advisor
    permissions TEXT, -- JSON: {"invoices": "rw", "tax": "r", ...}
    is_default INTEGER DEFAULT 0,
    invited_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (company_id) REFERENCES companies(id),
    UNIQUE(user_id, company_id)
);

-- Assets (company-scoped)
CREATE TABLE IF NOT EXISTS assets (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    asset_type TEXT CHECK(asset_type IN ('physical', 'virtual', 'data', 'intangible')) DEFAULT 'physical',
    status TEXT DEFAULT 'active',
    description TEXT,
    serial_number TEXT,
    location TEXT,
    purchase_date TEXT,
    purchase_price REAL,
    current_value REAL,
    expiry_date TEXT,
    category_id TEXT,
    image_url TEXT,
    -- 折旧字段
    useful_life INTEGER,                              -- 使用年限（月）
    salvage_rate REAL DEFAULT 0.05,                   -- 残值率
    depreciation_method TEXT DEFAULT 'straight_line',  -- straight_line | declining_balance | sum_of_years
    monthly_depreciation REAL,                        -- 月折旧额
    accumulated_depreciation REAL DEFAULT 0,           -- 累计折旧
    invoice_id TEXT,                                    -- 购入发票
    contract_id TEXT,                                   -- 购入合同
    registration_number TEXT,                           -- 登记号/专利号
    holder_name TEXT,                                   -- 持有人/权利人
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Asset Categories (company-scoped)
CREATE TABLE IF NOT EXISTS asset_categories (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    category_type TEXT DEFAULT 'general',
    description TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Invoices (company-scoped)
-- invoice_type axis: receivable (AR/销项) | payable (AP/进项)
CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    invoice_number TEXT NOT NULL,
    invoice_type TEXT,
    amount REAL NOT NULL,
    tax_amount REAL DEFAULT 0,
    status TEXT DEFAULT 'pending',
    invoice_date TEXT,
    due_date TEXT,
    supplier TEXT,
    description TEXT,
    file_path TEXT,
    created_by TEXT,
    contract_id TEXT,                  -- 关联合同
    currency TEXT DEFAULT 'CNY',
    client_id TEXT,                    -- 关联客户（应收/销项）
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Employees (company-scoped)
CREATE TABLE IF NOT EXISTS employees (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    full_name TEXT,
    employee_number TEXT,
    email TEXT,
    phone TEXT,
    department TEXT,
    position TEXT,
    hire_date TEXT,
    status TEXT DEFAULT 'active',
    employment_type TEXT DEFAULT 'full_time',
    base_salary REAL DEFAULT 0,
    social_security_base REAL DEFAULT 0,
    housing_fund_base REAL DEFAULT 0,
    notes TEXT,
    avatar_url TEXT,
    contract_start_date TEXT,
    contract_end_date TEXT,
    bank_account TEXT,
    bank_name TEXT,
    emergency_contact TEXT,
    emergency_phone TEXT,
    id_number TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

CREATE TABLE IF NOT EXISTS employee_documents (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    employee_id TEXT NOT NULL,
    document_type TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    description TEXT,
    upload_date TEXT DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (employee_id) REFERENCES employees(id)
);

-- Budgets (company-scoped)
CREATE TABLE IF NOT EXISTS budgets (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    category TEXT NOT NULL,
    budget_type TEXT NOT NULL,
    period TEXT NOT NULL,
    budget_amount REAL DEFAULT 0,
    actual_amount REAL DEFAULT 0,
    status TEXT DEFAULT 'active',
    project_id TEXT,                   -- 关联项目
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Bank Accounts (company-scoped)
CREATE TABLE IF NOT EXISTS bank_accounts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    bank_name TEXT NOT NULL,
    account_name TEXT NOT NULL,
    account_number TEXT NOT NULL,
    account_type TEXT DEFAULT 'checking',
    balance REAL DEFAULT 0,
    currency TEXT DEFAULT 'CNY',
    status TEXT DEFAULT 'active',
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Bank Transactions (company-scoped)
CREATE TABLE IF NOT EXISTS bank_transactions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    transaction_type TEXT NOT NULL,
    amount REAL NOT NULL,
    balance_after REAL,
    counterparty TEXT,
    description TEXT,
    transaction_date TEXT NOT NULL,
    payment_id TEXT,                   -- 关联付款记录
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (account_id) REFERENCES bank_accounts(id)
);

-- Contracts (company-scoped)
CREATE TABLE IF NOT EXISTS contracts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    contract_name TEXT NOT NULL,
    contract_number TEXT,
    contract_type TEXT,
    counterparty TEXT,
    party_a TEXT,
    party_b TEXT,
    amount REAL,
    start_date TEXT,
    end_date TEXT,
    status TEXT DEFAULT 'active',
    file_path TEXT,
    project_id TEXT,
    content TEXT,
    notes TEXT,
    ai_review_status TEXT DEFAULT 'pending',  -- 'pending', 'reviewing', 'completed', 'failed', 'unavailable'
    ai_risk_level TEXT,                       -- '🔴 high', '🟡 medium', '🟢 low'
    ai_review_result TEXT,                    -- Full markdown review from AI
    ai_reviewed_at TEXT,                      -- Timestamp of last review
    counterparty_type TEXT,                   -- 'vc', 'angel', 'government', 'incubator', 'employee', 'agency', 'client', 'landlord'
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Projects (company-scoped)
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    project_type TEXT,
    leader_name TEXT,
    status TEXT DEFAULT 'active',
    priority TEXT DEFAULT 'medium',
    budget REAL DEFAULT 0,
    spent REAL DEFAULT 0,
    start_date TEXT,
    end_date TEXT,
    client TEXT,
    progress INTEGER DEFAULT 0,
    team_members INTEGER DEFAULT 0,
    goals TEXT,
    notes TEXT,
    milestones TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

CREATE TABLE IF NOT EXISTS project_documents (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    document_type TEXT NOT NULL,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    description TEXT,
    upload_date TEXT DEFAULT CURRENT_TIMESTAMP,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Payments (company-scoped)
CREATE TABLE IF NOT EXISTS payments (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    payment_no TEXT,
    payment_type TEXT NOT NULL,
    payment_method TEXT,
    amount REAL NOT NULL,
    currency TEXT DEFAULT 'CNY',
    category TEXT,
    status TEXT DEFAULT 'pending',
    payee TEXT,
    payer TEXT,
    bank_account_id TEXT,
    invoice_id TEXT,
    contract_id TEXT,
    salary_record_id TEXT,
    rent_record_id TEXT,
    description TEXT,
    notes TEXT,
    payment_date TEXT,
    due_date TEXT,
    invoice_required INTEGER DEFAULT 0,
    invoice_deadline TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Salary Records (company-scoped)
CREATE TABLE IF NOT EXISTS salary_records (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    employee_id TEXT NOT NULL,
    employee_name TEXT,
    department TEXT,
    position TEXT,
    period TEXT NOT NULL,
    base_salary REAL DEFAULT 0,
    bonus REAL DEFAULT 0,
    deductions REAL DEFAULT 0,
    deduction REAL DEFAULT 0,
    social_security REAL DEFAULT 0,
    social_security_company REAL DEFAULT 0,
    housing_fund REAL DEFAULT 0,
    housing_fund_company REAL DEFAULT 0,
    net_salary REAL DEFAULT 0,
    status TEXT DEFAULT 'pending',
    payment_date TEXT,
    payment_id TEXT,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (employee_id) REFERENCES employees(id)
);

-- Rent Records (company-scoped)
CREATE TABLE IF NOT EXISTS rent_records (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    location_name TEXT NOT NULL,
    property_type TEXT DEFAULT 'office',
    address TEXT,
    area REAL DEFAULT 0,
    monthly_rent REAL NOT NULL,
    deposit REAL DEFAULT 0,
    contract_start_date TEXT,
    contract_end_date TEXT,
    next_payment_date TEXT,
    payment_cycle TEXT DEFAULT 'monthly',
    landlord TEXT,
    landlord_contact TEXT,
    contract_id TEXT,
    status TEXT DEFAULT 'active',
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Licenses (company-scoped) - 企业证照管理
CREATE TABLE IF NOT EXISTS licenses (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    license_name TEXT NOT NULL,
    license_type TEXT DEFAULT 'other',
    license_number TEXT,
    issuing_authority TEXT,
    issue_date TEXT,
    expiry_date TEXT,
    file_path TEXT,
    status TEXT DEFAULT 'valid',
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Reminders (company-scoped)
CREATE TABLE IF NOT EXISTS reminders (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    reminder_type TEXT DEFAULT 'general',
    priority TEXT DEFAULT 'medium',
    due_date TEXT,
    remind_date TEXT,
    status TEXT DEFAULT 'pending',
    is_completed INTEGER DEFAULT 0,
    entity_type TEXT,
    entity_id TEXT,
    notified_at TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- KPI Metrics (company-scoped)
CREATE TABLE IF NOT EXISTS kpi_metrics (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    category TEXT,
    value REAL DEFAULT 0,
    target REAL DEFAULT 0,
    unit TEXT,
    period TEXT,
    description TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Tax Records (company-scoped)
CREATE TABLE IF NOT EXISTS tax_records (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    tax_type TEXT NOT NULL,
    period TEXT NOT NULL,
    declaration_type TEXT,
    declaration_period TEXT,
    period_start TEXT,
    period_end TEXT,
    taxable_amount REAL DEFAULT 0,
    tax_amount REAL DEFAULT 0,
    declared_amount REAL DEFAULT 0,
    status TEXT DEFAULT 'pending',
    due_date TEXT,
    declaration_date TEXT,
    declaration_deadline TEXT,
    payment_date TEXT,
    notes TEXT,
    invoice_id TEXT,
    payment_id TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Operation Logs (company-scoped)
-- Contract Review History (审查版本记录)
CREATE TABLE IF NOT EXISTS contract_review_history (
    id TEXT PRIMARY KEY,
    contract_id TEXT NOT NULL,
    company_id TEXT NOT NULL,
    risk_level TEXT,
    review_result TEXT,
    counterparty_type TEXT,
    reviewed_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (contract_id) REFERENCES contracts(id)
);

CREATE TABLE IF NOT EXISTS operation_logs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    action TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    details TEXT,
    ip_address TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Error Logs (前端错误自动上报 + 后端 5xx 中间件落库)
CREATE TABLE IF NOT EXISTS error_logs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    created_by TEXT,
    error_type TEXT NOT NULL DEFAULT 'unknown',
    error_message TEXT NOT NULL,
    stack_trace TEXT,
    component_stack TEXT,
    page_url TEXT,
    user_agent TEXT,
    metadata TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

-- App-level key/value (health write probe, feature flags, …)
CREATE TABLE IF NOT EXISTS app_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Files (attachments for invoices, contracts, etc.)
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    filename TEXT NOT NULL,
    original_name TEXT NOT NULL,
    content_type TEXT DEFAULT 'application/octet-stream',
    size INTEGER DEFAULT 0,
    entity_type TEXT, -- 'invoice', 'contract', 'asset', etc.
    entity_id TEXT,
    description TEXT,
    uploaded_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    -- DaaS document-fusion mirror reference (Phase 1). Local FS stays canonical
    -- for serving; these point at the governed copy in DaaS.
    daas_workspace_id TEXT,
    daas_document_asset_id TEXT,
    daas_object_key TEXT,
    daas_sync_status TEXT DEFAULT 'pending', -- pending | synced | skipped | failed
    daas_synced_at TEXT,
    daas_sync_error TEXT,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (uploaded_by) REFERENCES users(id)
);

-- DaaS workspace/project mapping per ERP company (Phase 1 tenancy bridge).
-- A DaaS workspace_id == project_id (1:1); lazily created on first upload.
CREATE TABLE IF NOT EXISTS daas_company_workspace (
    company_id TEXT PRIMARY KEY,
    daas_workspace_id TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Sessions (for OAuth)
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    access_token TEXT,
    refresh_token TEXT,
    current_company_id TEXT,
    expires_at TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (current_company_id) REFERENCES companies(id)
);

-- API Keys (for third-party integration / SDK / MCP)
CREATE TABLE IF NOT EXISTS api_keys (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,                -- Human-readable label e.g. "Claude Desktop"
    key_hash TEXT NOT NULL UNIQUE,     -- SHA-256 hash of the actual key
    key_prefix TEXT NOT NULL,          -- First 8 chars for identification (om_xxxx...)
    scopes TEXT DEFAULT '*',           -- Comma-separated scopes: * | read | write | contracts | invoices
    rate_limit INTEGER DEFAULT 1000,   -- Requests per hour
    last_used_at TEXT,
    expires_at TEXT,                   -- NULL = never expires
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Agent execution audit logs (REST agent gateway / MCP)
CREATE TABLE IF NOT EXISTS agent_audit_logs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    api_key_id TEXT,
    agent_name TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    scope TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    request_payload TEXT NOT NULL,
    response_payload TEXT,
    status TEXT NOT NULL,
    error_message TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (api_key_id) REFERENCES api_keys(id)
);
CREATE INDEX IF NOT EXISTS idx_agent_audit_logs_company_created ON agent_audit_logs(company_id, created_at);
CREATE INDEX IF NOT EXISTS idx_agent_audit_logs_tool ON agent_audit_logs(company_id, tool_name);

-- Data Asset Metadata (数据资产详情)
CREATE TABLE IF NOT EXISTS data_asset_meta (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    asset_id TEXT NOT NULL UNIQUE,
    data_type TEXT NOT NULL,           -- software|patent|trademark|domain|dataset|copyright|trade_secret
    tech_stack TEXT,                   -- 技术栈（软件类）
    version TEXT,                      -- 版本号
    license_type TEXT,                 -- 开源/商业/自研
    patent_number TEXT,                -- 专利号
    patent_type TEXT,                  -- 发明/实用新型/外观
    trademark_class TEXT,              -- 商标分类
    data_volume TEXT,                  -- 数据量（数据集类）
    data_format TEXT,                  -- 数据格式
    access_url TEXT,                   -- 访问地址
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (asset_id) REFERENCES assets(id)
);

-- R&D Costs (研发费用归集)
CREATE TABLE IF NOT EXISTS rd_costs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    cost_type TEXT NOT NULL,           -- personnel|equipment|material|outsourcing|depreciation|other
    amount REAL NOT NULL,
    description TEXT,
    cost_date TEXT,
    employee_id TEXT,                  -- 关联员工（人员费用）
    asset_id TEXT,                     -- 关联设备（设备折旧）
    invoice_id TEXT,                   -- 关联发票
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Asset Records (资产变动记录)
CREATE TABLE IF NOT EXISTS asset_records (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    asset_id TEXT NOT NULL,
    record_type TEXT NOT NULL,
    description TEXT,
    amount REAL,
    record_date TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (asset_id) REFERENCES assets(id)
);

-- Cloud Bills (云账单)
CREATE TABLE IF NOT EXISTS cloud_bills (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    provider_id TEXT,
    billing_period TEXT NOT NULL,
    total_amount REAL DEFAULT 0,
    currency TEXT DEFAULT 'CNY',
    status TEXT DEFAULT 'pending',
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_company_members_user ON company_members(user_id);
CREATE INDEX IF NOT EXISTS idx_company_members_company ON company_members(company_id);
CREATE INDEX IF NOT EXISTS idx_assets_company ON assets(company_id);
CREATE INDEX IF NOT EXISTS idx_invoices_company ON invoices(company_id);
CREATE INDEX IF NOT EXISTS idx_employees_company ON employees(company_id);
CREATE INDEX IF NOT EXISTS idx_budgets_company ON budgets(company_id);
CREATE INDEX IF NOT EXISTS idx_bank_accounts_company ON bank_accounts(company_id);
CREATE INDEX IF NOT EXISTS idx_contracts_company ON contracts(company_id);
CREATE INDEX IF NOT EXISTS idx_projects_company ON projects(company_id);
CREATE INDEX IF NOT EXISTS idx_payments_company ON payments(company_id);
CREATE INDEX IF NOT EXISTS idx_salary_records_company ON salary_records(company_id);
CREATE INDEX IF NOT EXISTS idx_rent_records_company ON rent_records(company_id);
CREATE INDEX IF NOT EXISTS idx_licenses_company ON licenses(company_id);
CREATE INDEX IF NOT EXISTS idx_reminders_company ON reminders(company_id);
CREATE INDEX IF NOT EXISTS idx_kpi_metrics_company ON kpi_metrics(company_id);
CREATE INDEX IF NOT EXISTS idx_tax_records_company ON tax_records(company_id);
CREATE INDEX IF NOT EXISTS idx_operation_logs_company ON operation_logs(company_id);
CREATE INDEX IF NOT EXISTS idx_error_logs_company ON error_logs(company_id);
CREATE INDEX IF NOT EXISTS idx_error_logs_type ON error_logs(error_type);
CREATE INDEX IF NOT EXISTS idx_error_logs_created ON error_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_data_asset_meta_company ON data_asset_meta(company_id);
CREATE INDEX IF NOT EXISTS idx_data_asset_meta_asset ON data_asset_meta(asset_id);
CREATE INDEX IF NOT EXISTS idx_rd_costs_company ON rd_costs(company_id);
CREATE INDEX IF NOT EXISTS idx_rd_costs_project ON rd_costs(project_id);
CREATE INDEX IF NOT EXISTS idx_rd_costs_employee ON rd_costs(employee_id);
CREATE INDEX IF NOT EXISTS idx_rd_costs_asset ON rd_costs(asset_id);
CREATE INDEX IF NOT EXISTS idx_rd_costs_invoice ON rd_costs(invoice_id);
CREATE INDEX IF NOT EXISTS idx_contracts_project ON contracts(project_id);
CREATE INDEX IF NOT EXISTS idx_invoices_contract ON invoices(contract_id);
CREATE INDEX IF NOT EXISTS idx_budgets_project ON budgets(project_id);
CREATE INDEX IF NOT EXISTS idx_payments_bank_account ON payments(bank_account_id);
CREATE INDEX IF NOT EXISTS idx_payments_invoice ON payments(invoice_id);
CREATE INDEX IF NOT EXISTS idx_payments_contract ON payments(contract_id);
CREATE INDEX IF NOT EXISTS idx_salary_records_payment ON salary_records(payment_id);
CREATE INDEX IF NOT EXISTS idx_rent_records_contract ON rent_records(contract_id);
CREATE INDEX IF NOT EXISTS idx_asset_records_company ON asset_records(company_id);
CREATE INDEX IF NOT EXISTS idx_asset_records_asset ON asset_records(asset_id);
CREATE INDEX IF NOT EXISTS idx_cloud_bills_company ON cloud_bills(company_id);
CREATE INDEX IF NOT EXISTS idx_cloud_bills_provider ON cloud_bills(provider_id);

-- Cloud Cost Management (H4: previously only in migration)
CREATE TABLE IF NOT EXISTS cloud_providers (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    provider_name TEXT NOT NULL,
    provider_type TEXT NOT NULL,
    account_name TEXT NOT NULL,
    account_id TEXT,
    region TEXT,
    contact_person TEXT,
    contact_email TEXT,
    contact_phone TEXT,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

CREATE TABLE IF NOT EXISTS cloud_resources (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    provider_id TEXT,
    resource_type TEXT NOT NULL,
    resource_name TEXT NOT NULL,
    instance_id TEXT,
    specification TEXT,
    region TEXT,
    purchase_type TEXT DEFAULT 'subscription',
    start_date TEXT NOT NULL,
    end_date TEXT,
    monthly_cost REAL DEFAULT 0,
    status TEXT DEFAULT 'running',
    notes TEXT,
    project_id TEXT,                    -- 关联项目
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

CREATE TABLE IF NOT EXISTS domain_subscriptions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    domain_name TEXT NOT NULL,
    registrar TEXT NOT NULL,
    registration_date TEXT NOT NULL,
    expiry_date TEXT NOT NULL,
    initial_purchase_cost REAL DEFAULT 0,
    annual_fee REAL DEFAULT 0,
    auto_renewal INTEGER DEFAULT 0,
    dns_provider TEXT,
    status TEXT DEFAULT 'active',
    contact_email TEXT,
    notes TEXT,
    payment_id TEXT,                    -- 关联付款
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

CREATE TABLE IF NOT EXISTS software_subscriptions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    software_name TEXT NOT NULL,
    vendor TEXT NOT NULL,
    subscription_type TEXT NOT NULL,
    plan_name TEXT,
    license_count INTEGER DEFAULT 1,
    total_price REAL DEFAULT 0,
    billing_cycle TEXT NOT NULL,
    start_date TEXT NOT NULL,
    next_billing_date TEXT NOT NULL,
    auto_renewal INTEGER DEFAULT 1,
    status TEXT DEFAULT 'active',
    usage_department TEXT,
    account_email TEXT,
    notes TEXT,
    payment_id TEXT,                    -- 关联付款
    contract_id TEXT,                   -- 关联合同
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

CREATE INDEX IF NOT EXISTS idx_cloud_providers_company ON cloud_providers(company_id);
CREATE INDEX IF NOT EXISTS idx_cloud_resources_company ON cloud_resources(company_id);
CREATE INDEX IF NOT EXISTS idx_domain_subscriptions_company ON domain_subscriptions(company_id);
CREATE INDEX IF NOT EXISTS idx_software_subscriptions_company ON software_subscriptions(company_id);
CREATE INDEX IF NOT EXISTS idx_bank_tx_payment ON bank_transactions(payment_id);
CREATE INDEX IF NOT EXISTS idx_tax_records_invoice ON tax_records(invoice_id);
CREATE INDEX IF NOT EXISTS idx_tax_records_payment ON tax_records(payment_id);
CREATE INDEX IF NOT EXISTS idx_cloud_resources_project ON cloud_resources(project_id);
CREATE INDEX IF NOT EXISTS idx_domain_subs_payment ON domain_subscriptions(payment_id);
CREATE INDEX IF NOT EXISTS idx_sw_subs_payment ON software_subscriptions(payment_id);
CREATE INDEX IF NOT EXISTS idx_sw_subs_contract ON software_subscriptions(contract_id);
CREATE INDEX IF NOT EXISTS idx_assets_invoice ON assets(invoice_id);
CREATE INDEX IF NOT EXISTS idx_assets_contract ON assets(contract_id);

-- Vendor/Partner Management (友商资料)
CREATE TABLE IF NOT EXISTS vendors (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    short_name TEXT,
    unified_code TEXT,
    legal_person TEXT,
    registered_capital TEXT,
    establishment_date TEXT,
    business_scope TEXT,
    address TEXT,
    contact_person TEXT,
    contact_phone TEXT,
    contact_email TEXT,
    bank_name TEXT,
    bank_account TEXT,
    vendor_type TEXT DEFAULT 'supplier',
    status TEXT DEFAULT 'active',
    notes TEXT,
    license_file_path TEXT,
    license_file_name TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_vendors_company ON vendors(company_id);

-- KPI Configs (KPI指标配置)
CREATE TABLE IF NOT EXISTS kpi_configs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    category TEXT,
    description TEXT,
    unit TEXT,
    target_value REAL,
    warning_threshold REAL,
    danger_threshold REAL,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- KPI Records (KPI记录)
CREATE TABLE IF NOT EXISTS kpi_records (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    kpi_id TEXT NOT NULL,
    value REAL DEFAULT 0,
    period TEXT NOT NULL,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (kpi_id) REFERENCES kpi_configs(id)
);

-- Applications (项目申报)
CREATE TABLE IF NOT EXISTS applications (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    application_no TEXT,
    title TEXT NOT NULL,
    type TEXT NOT NULL,
    category TEXT,
    deadline TEXT,
    status TEXT DEFAULT 'draft',
    progress INTEGER DEFAULT 0,
    notes TEXT,
    project_id TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Application Materials (申报材料)
CREATE TABLE IF NOT EXISTS application_materials (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    application_id TEXT NOT NULL,
    name TEXT NOT NULL,
    required INTEGER DEFAULT 1,
    status TEXT DEFAULT 'pending',
    description TEXT,
    upload_date TEXT,
    file_size TEXT,
    file_path TEXT,
    can_reuse INTEGER DEFAULT 0,
    reuse_source TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (application_id) REFERENCES applications(id)
);

-- Global Policies Database (全球政策库)
CREATE TABLE IF NOT EXISTS policies (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    name_en TEXT,
    name_local TEXT,
    country TEXT NOT NULL,
    region TEXT,
    category TEXT NOT NULL,
    subcategory TEXT,
    provider TEXT,
    description TEXT,
    eligibility TEXT,
    benefits TEXT,
    amount_min REAL,
    amount_max REAL,
    currency TEXT DEFAULT 'USD',
    equity_required REAL,
    deadline TEXT,
    cycle TEXT,
    url TEXT,
    match_industries TEXT,
    match_min_employees INTEGER,
    match_max_employees INTEGER,
    match_min_revenue REAL,
    match_max_revenue REAL,
    match_countries TEXT,
    match_company_age_max INTEGER,
    match_founder_education TEXT,
    match_founder_is_expat INTEGER,
    match_founder_is_overseas_returnee INTEGER,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Policy Applications (政策申报追踪)
CREATE TABLE IF NOT EXISTS policy_applications (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    policy_id TEXT NOT NULL,
    status TEXT DEFAULT 'interested',
    applied_date TEXT,
    result_date TEXT,
    result_amount REAL,
    notes TEXT,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (policy_id) REFERENCES policies(id)
);

-- Material Library (材料库)
CREATE TABLE IF NOT EXISTS material_library (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    category TEXT,
    upload_date TEXT,
    file_size TEXT,
    file_path TEXT,
    used_count INTEGER DEFAULT 0,
    source_type TEXT,
    source_id TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Material Templates (材料模板)
CREATE TABLE IF NOT EXISTS material_templates (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    template_type TEXT,
    template_name TEXT NOT NULL,
    template_category TEXT,
    material_name TEXT,
    required INTEGER DEFAULT 0,
    description TEXT,
    display_order INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Material Versions (材料版本)
CREATE TABLE IF NOT EXISTS material_versions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    material_id TEXT,
    version_number INTEGER DEFAULT 1,
    file_path TEXT,
    file_name TEXT,
    file_size TEXT,
    description TEXT,
    change_notes TEXT,
    created_by TEXT,
    is_current INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Branches (分支机构)
CREATE TABLE IF NOT EXISTS branches (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    branch_name TEXT NOT NULL,
    branch_code TEXT,
    branch_type TEXT,
    legal_representative TEXT,
    unified_social_credit_code TEXT,
    registered_capital TEXT,
    contact_person TEXT,
    contact_phone TEXT,
    contact_email TEXT,
    branch_address TEXT,
    business_scope TEXT,
    registration_date TEXT,
    employee_count INTEGER DEFAULT 0,
    parent_branch_id TEXT,
    status TEXT DEFAULT 'active',
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Milestones (里程碑)
CREATE TABLE IF NOT EXISTS milestones (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    milestone_type TEXT,
    importance TEXT DEFAULT 'medium',
    milestone_date TEXT,
    project_id TEXT,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Company Info (企业详情)
CREATE TABLE IF NOT EXISTS company_info (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    company_name TEXT NOT NULL,
    legal_representative TEXT,
    unified_social_credit_code TEXT,
    registration_date TEXT,
    registered_capital REAL,
    taxpayer_type TEXT DEFAULT 'small_scale',
    tax_registration_number TEXT,
    tax_authority TEXT,
    vat_rate REAL DEFAULT 3,
    invoice_limit REAL,
    cash_flow_threshold REAL DEFAULT 50000,
    contact_phone TEXT,
    contact_email TEXT,
    founder_education TEXT,
    founder_is_expat INTEGER DEFAULT 0,
    founder_is_overseas_returnee INTEGER DEFAULT 0,
    company_address TEXT,
    business_scope TEXT,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Utilities (水电费)
CREATE TABLE IF NOT EXISTS utilities (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    location_name TEXT NOT NULL,
    utility_type TEXT NOT NULL,
    period TEXT NOT NULL,
    usage REAL,
    unit_price REAL,
    amount REAL,
    due_date TEXT,
    status TEXT DEFAULT 'unpaid',
    notes TEXT,
    payment_id TEXT,
    rent_record_id TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Profiles (用户档案)
CREATE TABLE IF NOT EXISTS profiles (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    user_id TEXT,
    display_name TEXT,
    avatar_url TEXT,
    phone TEXT,
    department TEXT,
    position TEXT,
    email TEXT,
    bio TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- User Roles (用户角色)
CREATE TABLE IF NOT EXISTS user_roles (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    user_id TEXT,
    role TEXT,
    permissions TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Social Insurance Configs (社保配置)
CREATE TABLE IF NOT EXISTS social_insurance_configs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    city TEXT,
    social_security_base_min REAL,
    social_security_base_max REAL,
    housing_fund_base_min REAL,
    housing_fund_base_max REAL,
    pension_personal_rate REAL,
    pension_company_rate REAL,
    medical_personal_rate REAL,
    medical_company_rate REAL,
    unemployment_personal_rate REAL,
    unemployment_company_rate REAL,
    work_injury_company_rate REAL,
    maternity_company_rate REAL,
    housing_fund_personal_rate REAL,
    housing_fund_company_rate REAL,
    notes TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- AI Configurations (AI配置)
CREATE TABLE IF NOT EXISTS ai_configurations (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    provider TEXT,
    model TEXT,
    api_key TEXT,
    endpoint TEXT,
    enabled INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Email Configurations (邮件配置)
CREATE TABLE IF NOT EXISTS email_configurations (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    smtp_host TEXT,
    smtp_port INTEGER,
    smtp_user TEXT,
    smtp_password TEXT,
    sender_name TEXT,
    sender_email TEXT,
    enabled INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Inbound XML e-invoice mailboxes (IMAP TLS only)
CREATE TABLE IF NOT EXISTS invoice_mailboxes (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    imap_host TEXT NOT NULL,
    imap_port INTEGER NOT NULL DEFAULT 993,
    imap_username TEXT NOT NULL,
    imap_password TEXT NOT NULL,
    folder TEXT NOT NULL DEFAULT 'INBOX',
    enabled INTEGER NOT NULL DEFAULT 0,
    last_uid INTEGER NOT NULL DEFAULT 0,
    last_sync_at TEXT,
    last_sync_status TEXT,
    last_sync_error TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_invoice_mailboxes_company
    ON invoice_mailboxes(company_id, enabled);

-- One row per XML attachment observed through an invoice mailbox.
CREATE TABLE IF NOT EXISTS invoice_mail_imports (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    mailbox_id TEXT NOT NULL,
    message_uid INTEGER NOT NULL,
    message_id TEXT,
    subject TEXT,
    attachment_index INTEGER NOT NULL,
    attachment_name TEXT NOT NULL,
    content_sha256 TEXT NOT NULL,
    xml_content BLOB NOT NULL,
    invoice_number TEXT,
    parsed_json TEXT,
    status TEXT NOT NULL DEFAULT 'pending_confirmation'
        CHECK(status IN ('pending_confirmation', 'imported', 'duplicate', 'failed', 'rejected')),
    error TEXT,
    duplicate_of TEXT,
    invoice_id TEXT,
    confirmed_by TEXT,
    confirmed_at TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (mailbox_id) REFERENCES invoice_mailboxes(id),
    FOREIGN KEY (invoice_id) REFERENCES invoices(id),
    UNIQUE(company_id, mailbox_id, message_uid, attachment_index)
);
CREATE INDEX IF NOT EXISTS idx_invoice_mail_imports_queue
    ON invoice_mail_imports(company_id, status, created_at);
CREATE INDEX IF NOT EXISTS idx_invoice_mail_imports_hash
    ON invoice_mail_imports(company_id, content_sha256);

-- Additional indexes for new tables
CREATE INDEX IF NOT EXISTS idx_kpi_configs_company ON kpi_configs(company_id);
CREATE INDEX IF NOT EXISTS idx_kpi_records_company ON kpi_records(company_id);
CREATE INDEX IF NOT EXISTS idx_kpi_records_kpi ON kpi_records(kpi_id);
CREATE INDEX IF NOT EXISTS idx_applications_company ON applications(company_id);
CREATE INDEX IF NOT EXISTS idx_application_materials_company ON application_materials(company_id);
CREATE INDEX IF NOT EXISTS idx_application_materials_app ON application_materials(application_id);
CREATE INDEX IF NOT EXISTS idx_material_library_company ON material_library(company_id);
CREATE INDEX IF NOT EXISTS idx_material_templates_company ON material_templates(company_id);
CREATE INDEX IF NOT EXISTS idx_branches_company ON branches(company_id);
CREATE INDEX IF NOT EXISTS idx_milestones_company ON milestones(company_id);
CREATE INDEX IF NOT EXISTS idx_company_info_company ON company_info(company_id);
CREATE INDEX IF NOT EXISTS idx_utilities_company ON utilities(company_id);
CREATE INDEX IF NOT EXISTS idx_profiles_company ON profiles(company_id);
CREATE INDEX IF NOT EXISTS idx_user_roles_company ON user_roles(company_id);
CREATE INDEX IF NOT EXISTS idx_social_insurance_config_company ON social_insurance_configs(company_id);

-- G-6: Exchange Rates (汇率表)
CREATE TABLE IF NOT EXISTS exchange_rates (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    from_currency TEXT NOT NULL,    -- e.g. 'USD'
    to_currency TEXT NOT NULL,      -- e.g. 'CNY'
    rate REAL NOT NULL,             -- 1 USD = 7.25 CNY
    effective_date TEXT NOT NULL,   -- date this rate applies
    source TEXT DEFAULT 'manual',   -- 'manual', 'api', etc.
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_company ON exchange_rates(company_id);
CREATE INDEX IF NOT EXISTS idx_exchange_rates_pair ON exchange_rates(from_currency, to_currency, effective_date);

-- Global OPC Hubs (全球单人公司出海基准库)
CREATE TABLE IF NOT EXISTS global_opc_hubs (
    id TEXT PRIMARY KEY,
    country_code TEXT NOT NULL,
    city TEXT NOT NULL,
    currency TEXT DEFAULT 'USD',
    corporate_tax_rate REAL,
    dividend_tax_rate REAL,
    capital_gains_tax REAL,
    pass_through_allowed INTEGER DEFAULT 0,
    zero_tax_on_reinvested INTEGER DEFAULT 0,
    foreign_income_exempt INTEGER DEFAULT 0,
    digital_nomad_visa INTEGER DEFAULT 0,
    col_index REAL,
    avg_coworking_desk REAL,
    avg_apartment_rent_1bed REAL,
    setup_cost REAL,
    annual_compliance_cost REAL,
    hub_notes TEXT,
    is_active INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- ═══════════════════════════════════════
-- CRM Module (客户关系管理)
-- ═══════════════════════════════════════

-- Clients (客户/潜在客户)
CREATE TABLE IF NOT EXISTS clients (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,                    -- 客户名称/公司名
    short_name TEXT,                       -- 简称
    client_type TEXT DEFAULT 'prospect',   -- prospect/active/inactive/churned
    industry TEXT,
    website TEXT,
    email TEXT,
    phone TEXT,
    address TEXT,
    city TEXT,
    country TEXT,
    tax_id TEXT,                           -- 税号/统一社会信用代码
    source TEXT,                           -- referral/website/cold_call/social_media/other
    notes TEXT,
    tags TEXT,                             -- JSON array: ["vip", "tech"]
    annual_revenue REAL,                   -- 预估年营收
    employee_count INTEGER,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_clients_company ON clients(company_id);
CREATE INDEX IF NOT EXISTS idx_clients_type ON clients(company_id, client_type);

-- Contacts (联系人 — 一个客户可以有多个联系人)
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    client_id TEXT,                        -- 关联客户
    name TEXT NOT NULL,
    title TEXT,                            -- 职位
    email TEXT,
    phone TEXT,
    mobile TEXT,
    department TEXT,
    is_primary INTEGER DEFAULT 0,          -- 主要联系人
    notes TEXT,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (client_id) REFERENCES clients(id)
);
CREATE INDEX IF NOT EXISTS idx_contacts_company ON contacts(company_id);
CREATE INDEX IF NOT EXISTS idx_contacts_client ON contacts(client_id);

-- Interactions (客户互动时间线)
CREATE TABLE IF NOT EXISTS interactions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    client_id TEXT NOT NULL,
    contact_id TEXT,                        -- 可选关联到具体联系人
    interaction_type TEXT NOT NULL,         -- call/email/meeting/note/task
    subject TEXT,
    description TEXT,
    interaction_date TEXT NOT NULL,
    duration_minutes INTEGER,              -- 通话/会议时长
    outcome TEXT,                          -- positive/neutral/negative
    next_action TEXT,                      -- 下一步行动
    next_action_date TEXT,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (client_id) REFERENCES clients(id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id)
);
CREATE INDEX IF NOT EXISTS idx_interactions_company ON interactions(company_id);
CREATE INDEX IF NOT EXISTS idx_interactions_client ON interactions(client_id);
CREATE INDEX IF NOT EXISTS idx_interactions_date ON interactions(company_id, interaction_date);

-- Deals (销售机会/交易)
CREATE TABLE IF NOT EXISTS deals (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    client_id TEXT NOT NULL,
    contact_id TEXT,
    title TEXT NOT NULL,
    description TEXT,
    stage TEXT DEFAULT 'lead',             -- lead/qualified/proposal/negotiation/won/lost
    amount REAL DEFAULT 0,                 -- 预计金额
    currency TEXT DEFAULT 'CNY',
    probability INTEGER DEFAULT 10,        -- 成交概率 0-100
    expected_close_date TEXT,
    actual_close_date TEXT,
    lost_reason TEXT,                      -- 丢单原因
    source TEXT,                           -- 来源
    assigned_to TEXT,                      -- 负责人
    tags TEXT,                             -- JSON array
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (client_id) REFERENCES clients(id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id)
);
CREATE INDEX IF NOT EXISTS idx_deals_company ON deals(company_id);
CREATE INDEX IF NOT EXISTS idx_deals_client ON deals(client_id);
CREATE INDEX IF NOT EXISTS idx_deals_stage ON deals(company_id, stage);

-- Quotes (报价单)
CREATE TABLE IF NOT EXISTS quotes (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    client_id TEXT NOT NULL,
    contact_id TEXT,
    deal_id TEXT,                          -- 可关联到销售机会
    quote_number TEXT NOT NULL,            -- 报价单号: QT-2024-001
    title TEXT,
    status TEXT DEFAULT 'draft',           -- draft/sent/accepted/rejected/expired/invoiced
    subtotal REAL DEFAULT 0,
    tax_rate REAL DEFAULT 0,
    tax_amount REAL DEFAULT 0,
    discount REAL DEFAULT 0,
    total REAL DEFAULT 0,
    currency TEXT DEFAULT 'CNY',
    valid_until TEXT,                      -- 报价有效期
    notes TEXT,
    terms TEXT,                            -- 条款
    invoice_id TEXT,                       -- 转化后关联的发票
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (client_id) REFERENCES clients(id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id),
    FOREIGN KEY (deal_id) REFERENCES deals(id)
);
CREATE INDEX IF NOT EXISTS idx_quotes_company ON quotes(company_id);
CREATE INDEX IF NOT EXISTS idx_quotes_client ON quotes(client_id);
CREATE INDEX IF NOT EXISTS idx_quotes_number ON quotes(company_id, quote_number);

-- Quote Items (报价单明细)
CREATE TABLE IF NOT EXISTS quote_items (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    quote_id TEXT NOT NULL,
    description TEXT NOT NULL,
    quantity REAL DEFAULT 1,
    unit TEXT,                             -- 单位: 个/小时/月/项
    unit_price REAL DEFAULT 0,
    discount_percent REAL DEFAULT 0,
    amount REAL DEFAULT 0,                 -- quantity * unit_price * (1 - discount)
    sort_order INTEGER DEFAULT 0,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (quote_id) REFERENCES quotes(id)
);
CREATE INDEX IF NOT EXISTS idx_quote_items_quote ON quote_items(quote_id);

-- ═══════════════════════════════════════
-- Time Tracking Module (时间追踪)
-- ═══════════════════════════════════════

-- Time Entries (工时记录)
CREATE TABLE IF NOT EXISTS time_entries (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    project_id TEXT,                       -- 关联项目
    client_id TEXT,                        -- 关联客户
    employee_id TEXT,                      -- 关联员工
    description TEXT NOT NULL,             -- 工作描述
    start_time TEXT,                       -- 开始时间 (ISO 8601)
    end_time TEXT,                         -- 结束时间
    duration_minutes INTEGER DEFAULT 0,    -- 时长(分钟)
    date TEXT NOT NULL,                    -- 工作日期
    is_billable INTEGER DEFAULT 1,         -- 是否可计费
    hourly_rate REAL DEFAULT 0,            -- 时薪
    amount REAL DEFAULT 0,                 -- 金额 = duration/60 * hourly_rate
    invoice_id TEXT,                       -- 关联发票(计费后)
    status TEXT DEFAULT 'logged',          -- logged/running/invoiced
    tags TEXT,                             -- JSON array
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (client_id) REFERENCES clients(id),
    FOREIGN KEY (employee_id) REFERENCES employees(id)
);
CREATE INDEX IF NOT EXISTS idx_time_entries_company ON time_entries(company_id);
CREATE INDEX IF NOT EXISTS idx_time_entries_project ON time_entries(company_id, project_id);
CREATE INDEX IF NOT EXISTS idx_time_entries_client ON time_entries(company_id, client_id);
CREATE INDEX IF NOT EXISTS idx_time_entries_date ON time_entries(company_id, date);

-- Time entry tags (工时标签) — child of time_entries, parallel to quotes/quote_items
CREATE TABLE IF NOT EXISTS time_entry_tags (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    time_entry_id TEXT NOT NULL,
    tag TEXT NOT NULL,                     -- 标签名
    color TEXT,                            -- 可选颜色
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (time_entry_id) REFERENCES time_entries(id)
);
CREATE INDEX IF NOT EXISTS idx_time_entry_tags_company ON time_entry_tags(company_id);
CREATE INDEX IF NOT EXISTS idx_time_entry_tags_entry ON time_entry_tags(company_id, time_entry_id);

-- ═══════════════════════════════════════
-- Recurring Invoice & Reminders Module (周期性发票 + 催款)
-- ═══════════════════════════════════════

-- Recurring Invoices (周期性发票模板)
CREATE TABLE IF NOT EXISTS recurring_invoices (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    client_id TEXT,                         -- 关联客户
    title TEXT NOT NULL,                    -- 模板标题
    description TEXT,
    amount REAL NOT NULL,                   -- 金额
    currency TEXT DEFAULT 'CNY',
    tax_rate REAL DEFAULT 0,
    frequency TEXT NOT NULL,               -- monthly/quarterly/yearly/weekly
    start_date TEXT NOT NULL,              -- 起始日期
    end_date TEXT,                          -- 结束日期(空=无限)
    next_invoice_date TEXT NOT NULL,        -- 下次生成日期
    day_of_month INTEGER DEFAULT 1,        -- 每月几号
    auto_send INTEGER DEFAULT 0,           -- 自动发送
    payment_terms_days INTEGER DEFAULT 30, -- 付款期限(天)
    status TEXT DEFAULT 'active',          -- active/paused/completed
    total_generated INTEGER DEFAULT 0,     -- 已生成次数
    last_generated_at TEXT,                -- 最后生成时间
    notes TEXT,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (client_id) REFERENCES clients(id)
);
CREATE INDEX IF NOT EXISTS idx_recurring_invoices_company ON recurring_invoices(company_id);
CREATE INDEX IF NOT EXISTS idx_recurring_invoices_next ON recurring_invoices(next_invoice_date, status);

-- Invoice Reminders (发票催款提醒)
CREATE TABLE IF NOT EXISTS invoice_reminders (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    invoice_id TEXT NOT NULL,               -- 关联发票
    client_id TEXT,                         -- 关联客户
    reminder_type TEXT NOT NULL,            -- before_due/on_due/overdue_1/overdue_7/overdue_30
    scheduled_date TEXT NOT NULL,           -- 计划发送日期
    sent_at TEXT,                           -- 实际发送时间
    status TEXT DEFAULT 'pending',          -- pending/sent/skipped/failed
    subject TEXT,                           -- 邮件主题
    message TEXT,                           -- 邮件内容
    channel TEXT DEFAULT 'email',           -- email/sms/in_app
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (invoice_id) REFERENCES invoices(id),
    FOREIGN KEY (client_id) REFERENCES clients(id)
);
CREATE INDEX IF NOT EXISTS idx_invoice_reminders_company ON invoice_reminders(company_id);
CREATE INDEX IF NOT EXISTS idx_invoice_reminders_invoice ON invoice_reminders(invoice_id);
CREATE INDEX IF NOT EXISTS idx_invoice_reminders_schedule ON invoice_reminders(scheduled_date, status);

-- ═══════════════════════════════════════
-- Compliance: Insurance & Pension (保险/年金/社保合规)
-- ═══════════════════════════════════════

-- Insurance Policies (保险单 — 商业险/健康险/财产险/责任险)
CREATE TABLE IF NOT EXISTS insurance_policies (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    policy_name TEXT NOT NULL,              -- 保单名称
    policy_number TEXT,                     -- 保单号
    insurance_type TEXT NOT NULL,           -- health/property/liability/workers_comp/professional/cyber/vehicle
    provider TEXT,                          -- 保险公司
    premium_amount REAL DEFAULT 0,          -- 保费
    premium_frequency TEXT DEFAULT 'monthly', -- monthly/quarterly/yearly
    coverage_amount REAL DEFAULT 0,         -- 保额
    deductible REAL DEFAULT 0,             -- 免赔额
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    renewal_date TEXT,                      -- 续保日期
    auto_renew INTEGER DEFAULT 0,
    status TEXT DEFAULT 'active',           -- active/expired/cancelled/pending
    beneficiary TEXT,                       -- 受益人
    country TEXT,                           -- 适用国家/地区
    notes TEXT,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_insurance_policies_company ON insurance_policies(company_id);
CREATE INDEX IF NOT EXISTS idx_insurance_policies_renewal ON insurance_policies(renewal_date, status);

-- Pension / Social Security Plans (年金/社保计划)
CREATE TABLE IF NOT EXISTS pension_plans (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    plan_name TEXT NOT NULL,                -- 计划名称 (国民年金/建康保险/401k)
    plan_type TEXT NOT NULL,                -- national_pension/health_insurance/employment_insurance/401k/ira/private_pension
    country TEXT DEFAULT 'KR',             -- KR/US/CN/JP
    provider TEXT,                          -- 管理机构
    employee_id TEXT,                       -- 关联员工
    contribution_employer REAL DEFAULT 0,   -- 雇主缴纳
    contribution_employee REAL DEFAULT 0,   -- 员工缴纳
    contribution_frequency TEXT DEFAULT 'monthly',
    start_date TEXT,
    end_date TEXT,
    vesting_period_months INTEGER,          -- 归属期(月)
    current_value REAL DEFAULT 0,           -- 当前价值
    status TEXT DEFAULT 'active',           -- active/suspended/terminated/vested
    notes TEXT,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (employee_id) REFERENCES employees(id)
);
CREATE INDEX IF NOT EXISTS idx_pension_plans_company ON pension_plans(company_id);
CREATE INDEX IF NOT EXISTS idx_pension_plans_type ON pension_plans(company_id, plan_type);

-- ═══════════════════════════════════════
-- Global Tax Calendar Module (全球税务日历)
-- ═══════════════════════════════════════

-- Tax Jurisdictions (税务管辖区 — 国家/州/省)
CREATE TABLE IF NOT EXISTS tax_jurisdictions (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    code TEXT NOT NULL,                     -- 代码: US, US-CA, CN, KR, JP
    name TEXT NOT NULL,                     -- 名称
    name_local TEXT,                        -- 本地语言名称
    country_code TEXT NOT NULL,             -- 国家代码 ISO 3166
    region_code TEXT,                       -- 州/省代码
    jurisdiction_type TEXT NOT NULL,        -- country/state/province/city
    parent_jurisdiction_id TEXT,            -- 父级管辖区
    currency TEXT DEFAULT 'USD',
    fiscal_year_start TEXT,                 -- 财政年度开始 (MM-DD)
    corporate_tax_rate REAL,                -- 企业所得税率
    personal_tax_rate REAL,                 -- 个人所得税率
    vat_rate REAL,                          -- 增值税率/消费税率
    sales_tax_rate REAL,                    -- 销售税率
    has_state_income_tax INTEGER DEFAULT 1,
    notes TEXT,
    is_active INTEGER DEFAULT 1,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_tax_jurisdictions_company ON tax_jurisdictions(company_id);
CREATE INDEX IF NOT EXISTS idx_tax_jurisdictions_code ON tax_jurisdictions(company_id, code);

-- Tax Deadlines / Calendar Events (税务截止日期/日历事件)
CREATE TABLE IF NOT EXISTS tax_deadlines (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    jurisdiction_id TEXT,                   -- 关联管辖区
    jurisdiction_code TEXT NOT NULL,        -- 管辖区代码 (冗余用于快速查询)
    tax_type TEXT NOT NULL,                -- corporate_income/personal_income/vat/sales_tax/payroll/estimated/withholding/property/consumption/social_security
    title TEXT NOT NULL,                    -- 事件标题
    title_local TEXT,                       -- 本地语言标题
    description TEXT,
    deadline_month INTEGER NOT NULL,        -- 月份 (1-12)
    deadline_day INTEGER NOT NULL,          -- 日期 (1-31)
    recurrence TEXT DEFAULT 'yearly',       -- yearly/quarterly/monthly
    quarter INTEGER,                        -- 季度 (1-4, 用于季度截止)
    is_estimated_payment INTEGER DEFAULT 0, -- 是否为预缴税款
    applies_to TEXT DEFAULT 'all',          -- all/corporation/individual/sole_proprietor
    fiscal_year_relative INTEGER DEFAULT 0, -- 是否相对于财政年度
    days_after_fiscal_year_end INTEGER,     -- 财年结束后天数
    penalty_info TEXT,                       -- 逾期罚款信息
    filing_url TEXT,                         -- 申报网站
    is_system INTEGER DEFAULT 0,            -- 系统预设(不可删除)
    is_active INTEGER DEFAULT 1,
    notes TEXT,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_tax_deadlines_company ON tax_deadlines(company_id);
CREATE INDEX IF NOT EXISTS idx_tax_deadlines_jurisdiction ON tax_deadlines(company_id, jurisdiction_code);
CREATE INDEX IF NOT EXISTS idx_tax_deadlines_month ON tax_deadlines(deadline_month, deadline_day);

-- Email Logs (邮件发送记录)
CREATE TABLE IF NOT EXISTS email_logs (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    to_address TEXT NOT NULL,
    subject TEXT NOT NULL,
    template TEXT,                          -- invoice_send/payment_reminder/quote_send/custom
    entity_type TEXT,                       -- invoices/quotes/recurring_invoices
    entity_id TEXT,
    status TEXT DEFAULT 'pending',          -- pending/sent/failed/bounced
    error TEXT,
    sent_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_email_logs_company ON email_logs(company_id);
CREATE INDEX IF NOT EXISTS idx_email_logs_status ON email_logs(company_id, status);

-- Webhooks (WP8: event notification system)
CREATE TABLE IF NOT EXISTS webhooks (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    url TEXT NOT NULL,                      -- Target URL to POST events to
    events TEXT NOT NULL,                   -- Comma-separated: invoice.created,payment.received,contract.expiring,*
    secret TEXT,                            -- HMAC signing secret
    is_active INTEGER DEFAULT 1,
    description TEXT,
    last_triggered_at TEXT,
    failure_count INTEGER DEFAULT 0,
    created_by TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_webhooks_company ON webhooks(company_id);

-- Schema migration version tracking (applied ALTER ids; source of truth is
-- INIT_SQL + versioned ALTER list in db/mod.rs — not a migrations/ folder).
CREATE TABLE IF NOT EXISTS schema_migrations (
    id TEXT PRIMARY KEY,
    applied_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Built-in backup metadata (issue #2)
CREATE TABLE IF NOT EXISTS backup_records (
    id TEXT PRIMARY KEY,
    file_name TEXT NOT NULL UNIQUE,
    size_bytes INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'success',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_backup_records_created ON backup_records(created_at);

-- Document number sequences (INV-YYYY-NNNN / QUO-YYYY-NNNN)
CREATE TABLE IF NOT EXISTS document_sequences (
    company_id TEXT NOT NULL,
    doc_type TEXT NOT NULL,                 -- INV / QUO
    year INTEGER NOT NULL,
    next_seq INTEGER NOT NULL DEFAULT 1,
    PRIMARY KEY (company_id, doc_type, year)
);

-- Scheduler run log (billing, expiry, etc.)
CREATE TABLE IF NOT EXISTS scheduler_runs (
    id TEXT PRIMARY KEY,
    scheduler_name TEXT NOT NULL,
    last_run_at TEXT NOT NULL,
    generated_count INTEGER DEFAULT 0,
    details TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_scheduler_runs_name ON scheduler_runs(scheduler_name, last_run_at);
-- Global runtime settings (KV; e.g. document parser / OCR backend)
CREATE TABLE IF NOT EXISTS runtime_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Notification delivery channels
CREATE TABLE IF NOT EXISTS notification_channels (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    name TEXT,
    channel_url TEXT NOT NULL,
    mode TEXT DEFAULT 'digest',
    enabled INTEGER DEFAULT 1,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
CREATE INDEX IF NOT EXISTS idx_notification_channels_company ON notification_channels(company_id);

-- Automated AR collection settings (company-scoped; defaults 1/7/30/60 days)
CREATE TABLE IF NOT EXISTS collection_settings (
    company_id TEXT PRIMARY KEY,
    enabled INTEGER NOT NULL DEFAULT 1,
    friendly_days INTEGER NOT NULL DEFAULT 1,
    formal_days INTEGER NOT NULL DEFAULT 7,
    final_days INTEGER NOT NULL DEFAULT 30,
    bad_debt_days INTEGER NOT NULL DEFAULT 60,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id)
);

-- Durable invoice × level × channel idempotency and delivery history.
CREATE TABLE IF NOT EXISTS collection_deliveries (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL,
    invoice_id TEXT NOT NULL,
    client_id TEXT,
    invoice_number TEXT NOT NULL,
    client_name TEXT,
    amount_due REAL NOT NULL DEFAULT 0,
    days_overdue INTEGER NOT NULL,
    level_code TEXT NOT NULL,
    level_label TEXT NOT NULL,
    threshold_days INTEGER NOT NULL,
    channel_id TEXT NOT NULL,
    channel_name TEXT,
    channel_mode TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    attempt_count INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    audit_user_id TEXT,
    last_attempt_at TEXT,
    sent_at TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES companies(id),
    FOREIGN KEY (invoice_id) REFERENCES invoices(id),
    UNIQUE(company_id, invoice_id, level_code, channel_id)
);
CREATE INDEX IF NOT EXISTS idx_collection_deliveries_company_created
    ON collection_deliveries(company_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_collection_deliveries_invoice
    ON collection_deliveries(company_id, invoice_id, threshold_days DESC);
CREATE INDEX IF NOT EXISTS idx_collection_deliveries_dispatch
    ON collection_deliveries(channel_id, channel_mode, status);

-- Per-user sidebar visibility and order (JSON blob; catalog is frontend-owned)
CREATE TABLE IF NOT EXISTS user_nav_preferences (
    user_id TEXT NOT NULL,
    company_id TEXT NOT NULL,
    preferences TEXT NOT NULL,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, company_id),
    FOREIGN KEY (company_id) REFERENCES companies(id)
);
"#;
