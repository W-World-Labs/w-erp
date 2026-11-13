/**
 * OPC API Client
 * Replaces Supabase SDK with local Rust backend
 */

import { createHttpClient } from "@w/http-web";

import { reportError } from "@/lib/errorReporter";

function resolveApiBase(): string {
  const configured = (import.meta.env.VITE_API_URL || "").replace(/\/$/, "");

  if (
    configured === "http://localhost:8987" &&
    typeof window !== "undefined" &&
    !["localhost", "127.0.0.1"].includes(window.location.hostname)
  ) {
    return "";
  }

  return configured;
}

const API_BASE = resolveApiBase();

// Authentication is server-owned. JavaScript only remembers whether /auth/me
// succeeded; the opaque session secret stays in an HttpOnly cookie.
let authenticatedSessionObserved = false;

export function hasAuthenticatedSession(): boolean {
  return authenticatedSessionObserved;
}

// API request core — now backed by the shared @w/http-web client, configured to
// reproduce OPC's exact request shape + error behaviour:
//   • baseUrl = API_BASE (raw concat: `${API_BASE}${endpoint}`)
//   • HttpOnly cookie credentials, Content-Type unless FormData
//   • non-2xx → bare `Error(text || HTTP <status>)` (unchanged class)
//   • 401   → clear observed auth state + redirect to /auth
//   • errors auto-reported via reportError (skip /error_logs to avoid recursion;
//     HTTP failures only — network throws propagate raw, as before)
//   • empty/204 body → undefined, else response.json()
const httpClient = createHttpClient({
  baseUrl: API_BASE,
  // Resolve the global `fetch` per call (matches the old client's bare
  // `fetch(...)`), rather than capturing it once at construction time.
  fetchImpl: (...args: Parameters<typeof fetch>) => globalThis.fetch(...args),
  credentials: "include",
  contentType: "non-formdata",
  contentTypeValue: "application/json",
  parse: "json",
  emptyBody: undefined,
  errorMessage: (info) => info.raw || `HTTP ${info.status}`,
  errorFactory: (_info, message) => new Error(message),
  onUnauthorized: () => {
    authenticatedSessionObserved = false;
    window.location.href = "/auth";
    throw new Error("Unauthorized");
  },
  onError: (err, info) => {
    // 自动上报 API 错误到数据库 (排除 error_logs 本身避免递归)
    if (info.kind === "http" && !info.path.includes("/error_logs")) {
      reportError({
        error_type: "api_error",
        error_message: (err instanceof Error ? err.message : String(err)).substring(0, 500),
        metadata: {
          endpoint: info.path,
          method: info.method,
          status: info.status,
        },
      });
    }
  },
});

// API request helper
function request<T>(
  endpoint: string,
  options: RequestInit = {},
): Promise<T> {
  return httpClient.request<T>(endpoint, options);
}

// API methods
export const api = {
  // GET request
  async get<T>(endpoint: string): Promise<T> {
    return request<T>(endpoint);
  },

  // POST request
  async post<T>(endpoint: string, data?: unknown): Promise<T> {
    const isFormData = data instanceof FormData;
    return request<T>(endpoint, {
      method: "POST",
      body: isFormData
        ? (data as BodyInit)
        : data
          ? JSON.stringify(data)
          : undefined,
    });
  },

  // PUT request
  async put<T>(endpoint: string, data: unknown): Promise<T> {
    return request<T>(endpoint, {
      method: "PUT",
      body: JSON.stringify(data),
    });
  },

  // DELETE request
  async delete<T>(endpoint: string): Promise<T> {
    return request<T>(endpoint, {
      method: "DELETE",
    });
  },

  async runAction<T>(action: BackendAction, data?: unknown): Promise<T> {
    const method = action.method;
    switch (method) {
      case "POST":
        return api.post<T>(action.endpoint, data ?? {});
      case "PUT":
        return api.put<T>(action.endpoint, data ?? {});
      case "DELETE":
        return api.delete<T>(action.endpoint);
      case "GET":
        return api.get<T>(action.endpoint);
      default:
        throw new Error(`Unsupported backend action method: ${method}`);
    }
  },
};

// Auth helpers
export const auth = {
  login() {
    window.location.href = `${API_BASE}/auth/login`;
  },

  logout() {
    authenticatedSessionObserved = false;
    const form = document.createElement("form");
    form.method = "POST";
    form.action = `${API_BASE}/auth/logout`;
    form.hidden = true;
    document.body.appendChild(form);
    form.submit();
  },

  async getCurrentUser() {
    const response = await fetch(`${API_BASE}/auth/me`, {
      credentials: "include",
      headers: { Accept: "application/json" },
    });
    if (!response.ok) {
      authenticatedSessionObserved = false;
      throw new Error(response.status === 401 ? "Unauthorized" : `HTTP ${response.status}`);
    }
    const user = (await response.json()) as {
      id: string;
      email: string;
      name?: string;
      avatar_url?: string;
    };
    authenticatedSessionObserved = true;
    return user;
  },

  isAuthenticated(): boolean {
    return authenticatedSessionObserved;
  },
};

// Resource APIs
export const assets = {
  list: () => api.get<{ data: Asset[]; count: number }>("/api/assets"),
  get: (id: string) => api.get<{ data: Asset }>(`/api/assets/${id}`),
  create: (data: CreateAssetInput) =>
    api.post<{ data: Asset }>("/api/assets", data),
  update: (id: string, data: CreateAssetInput) =>
    api.put<{ data: Asset }>(`/api/assets/${id}`, data),
  delete: (id: string) => api.delete(`/api/assets/${id}`),
};

export const invoices = {
  list: () => api.get<{ data: Invoice[]; count: number }>("/api/invoices"),
  get: (id: string) => api.get<Invoice>(`/api/invoices/${id}`),
  create: (data: CreateInvoiceInput) =>
    api.post<Invoice>("/api/invoices", data),
  update: (id: string, data: Partial<CreateInvoiceInput>) =>
    api.put<Invoice>(`/api/invoices/${id}`, data),
  delete: (id: string) => api.delete(`/api/invoices/${id}`),
};

export const employees = {
  list: () => api.get<{ data: Employee[]; count: number }>("/api/employees"),
  get: (id: string) => api.get<Employee>(`/api/employees/${id}`),
  create: (data: EmployeeInput) => api.post<Employee>("/api/employees", data),
  update: (id: string, data: EmployeeInput) =>
    api.put<Employee>(`/api/employees/${id}`, data),
  delete: (id: string) => api.delete(`/api/employees/${id}`),
};

export const budgets = {
  list: () => api.get<{ data: Budget[]; count: number }>("/api/budgets"),
  get: (id: string) => api.get<Budget>(`/api/budgets/${id}`),
  create: (data: CreateBudgetInput) => api.post<Budget>("/api/budgets", data),
  update: (id: string, data: Partial<CreateBudgetInput>) =>
    api.put<Budget>(`/api/budgets/${id}`, data),
  delete: (id: string) => api.delete(`/api/budgets/${id}`),
};

export const bankAccounts = {
  list: () =>
    api.get<{ data: BankAccount[]; count: number }>("/api/bank/accounts"),
  get: (id: string) => api.get<BankAccount>(`/api/bank/accounts/${id}`),
  create: (data: CreateBankAccountInput) =>
    api.post<BankAccount>("/api/bank/accounts", data),
  update: (id: string, data: Partial<CreateBankAccountInput>) =>
    api.put<BankAccount>(`/api/bank/accounts/${id}`, data),
  delete: (id: string) => api.delete(`/api/bank/accounts/${id}`),
};

// Company management
let currentCompanyId: string | null = null;

export function setCurrentCompany(id: string | null) {
  currentCompanyId = id;
  if (id) {
    localStorage.setItem("current_company_id", id);
  } else {
    localStorage.removeItem("current_company_id");
  }
  window.dispatchEvent(new Event("erp-company-changed"));
}

export function getCurrentCompanyId(): string | null {
  if (!currentCompanyId) {
    currentCompanyId = localStorage.getItem("current_company_id");
  }
  return currentCompanyId;
}

export interface Company {
  id: string;
  name: string;
  legal_name?: string;
  tax_id?: string;
  address?: string;
  phone?: string;
  role: string;
  role_badge?: CompanyMemberRoleBadge;
  is_default: boolean;
  is_current?: boolean;
}

export interface CompanyMember {
  id: string;
  user_id: string;
  email: string;
  name?: string;
  role: string;
}

export interface CompanyMemberRoleBadge {
  value: string;
  label: string;
  icon: string;
  variant: "default" | "secondary" | "outline" | "destructive";
}

export interface CompanyMemberOverviewItem extends CompanyMember {
  display_name: string;
  avatar_fallback: string;
  role_badge: CompanyMemberRoleBadge;
  can_edit_role: boolean;
  can_remove: boolean;
}

export interface CompanyMembersOverview {
  company: {
    id: string;
    name: string;
    current_user_role: string;
    is_admin: boolean;
  };
  stats: {
    total: number;
    owners: number;
    admins: number;
  };
  members: CompanyMemberOverviewItem[];
  assignable_roles: CompanyMemberRoleBadge[];
  empty_message: string;
}

export const companies = {
  // List companies the current user belongs to
  list: () => api.get<{ data: Company[] }>("/api/companies"),
  // Get company details
  get: (id: string) => api.get<Company>(`/api/companies/${id}`),
  // Create new company
  create: (data: {
    name: string;
    legal_name?: string;
    tax_id?: string;
    address?: string;
  }) => api.post<{ id: string; name: string }>("/api/companies", data),
  // Update company
  update: (id: string, data: Partial<Company>) =>
    api.put<{ id: string }>(`/api/companies/${id}`, data),
  // Delete company
  delete: (id: string) => api.delete(`/api/companies/${id}`),
  // Switch current company
  switch: async (id: string) => {
    const result = await api.post<{ current_company_id: string }>(
      `/api/companies/${id}/switch`,
      {},
    );
    setCurrentCompany(result.current_company_id);
    return result;
  },

  // Member management
  members: {
    list: (companyId: string) =>
      api.get<{ data: CompanyMember[] }>(`/api/companies/${companyId}/members`),
    overview: (companyId: string, search?: string) => {
      const params = new URLSearchParams();
      const value = search?.trim();
      if (value) params.set("search", value);
      const query = params.toString();
      return api.get<CompanyMembersOverview>(
        `/api/companies/${companyId}/members/overview${query ? `?${query}` : ""}`,
      );
    },
    add: (companyId: string, email: string, role?: string) =>
      api.post<{ id: string }>(`/api/companies/${companyId}/members`, {
        email,
        role,
      }),
    update: (companyId: string, memberId: string, role: string) =>
      api.put<{ id: string }>(
        `/api/companies/${companyId}/members/${memberId}`,
        { role },
      ),
    remove: (companyId: string, memberId: string) =>
      api.delete(`/api/companies/${companyId}/members/${memberId}`),
  },
};

// File upload API
export type BackendActionMethod = "GET" | "POST" | "PUT" | "DELETE";

export interface BackendAction {
  method: BackendActionMethod;
  endpoint: string;
  label?: string;
  confirm_message?: string | null;
  success_title?: string;
  success_message?: string;
  default_values?: Record<string, unknown>;
}

export interface DeleteAction extends BackendAction {
  method: "DELETE";
  confirm_message: string;
  success_message: string;
}

export interface FileInfo {
  id: string;
  filename: string;
  original_name: string;
  content_type: string;
  size: number;
  size_label?: string;
  created_at?: string;
  created_at_label?: string;
  file_name?: string;
  file_path?: string;
  file_type?: string;
  description?: string | null;
  upload_date?: string;
  upload_date_label?: string;
  file_icon?: string;
  file_icon_class_name?: string;
  delete_action?: DeleteAction | null;
  metadata_label?: string;
  url: string;
  entity_type?: string;
  entity_id?: string;
}

export const files = {
  // Upload a file
  upload: async (file: File): Promise<FileInfo> => {
    const formData = new FormData();
    formData.append("file", file);

    const response = await fetch(`${API_BASE}/api/files/upload`, {
      method: "POST",
      credentials: "include",
      body: formData,
    });

    if (!response.ok) {
      throw new Error(`Upload failed: ${response.status}`);
    }
    return response.json();
  },

  // Upload file for entity (invoice, contract, etc.)
  uploadForEntity: async (
    file: File,
    entityType: string,
    entityId: string,
    metadata?: { description?: string | null },
  ): Promise<FileInfo> => {
    const formData = new FormData();
    formData.append("file", file);
    if (metadata?.description?.trim()) {
      formData.append("description", metadata.description.trim());
    }

    const response = await fetch(
      `${API_BASE}/api/files/upload/${entityType}/${entityId}`,
      {
        method: "POST",
        credentials: "include",
        body: formData,
      },
    );

    if (!response.ok) {
      throw new Error(`Upload failed: ${response.status}`);
    }
    return response.json();
  },

  // List files for entity
  list: (entityType: string, entityId: string) =>
    api.get<{ data: FileInfo[] }>(`/api/files/list/${entityType}/${entityId}`),
  delete: (id: string) => api.delete(`/api/files/${encodeURIComponent(id)}`),
  // Get download URL
  getUrl: (id: string) => `${API_BASE}/api/files/${id}`,

  download: async (id: string): Promise<Blob> => {
    const response = await fetch(
      `${API_BASE}/api/files/${encodeURIComponent(id)}`,
      {
        credentials: "include",
      },
    );

    if (!response.ok) {
      throw new Error(`Download failed: ${response.status}`);
    }

    return response.blob();
  },
};

// AI API
export interface ChatMessage {
  role: "user" | "assistant" | "system";
  content: string;
}

export interface ChatResponse {
  reply: string;
  success: boolean;
}

export interface AnalyzeResponse {
  analysis: string;
  data_type: string;
  success: boolean;
}

export interface AuditResponse {
  audit_result: string;
  entity_type: string;
  success: boolean;
}

export const ai = {
  // Chat with AI assistant
  chat: (message: string, context?: string, history?: ChatMessage[]) =>
    api.post<ChatResponse>("/api/ai/chat", { message, context, history }),
  // Analyze data
  analyze: (dataType: string, data: unknown, question?: string) =>
    api.post<AnalyzeResponse>("/api/ai/analyze", {
      data_type: dataType,
      data,
      question,
    }),
  // Audit check
  audit: (entityType: string, entityData: unknown) =>
    api.post<AuditResponse>("/api/ai/audit", {
      entity_type: entityType,
      entity_data: entityData,
    }),
  // AI Contract Review (防坑卫士)
  contractReview: (
    contractText: string,
    contractType?: string,
    counterpartyType?: string,
  ) =>
    api.post<ContractReviewResponse>("/api/ai/contract-review", {
      contract_text: contractText,
      contract_type: contractType,
      counterparty_type: counterpartyType,
    }),
};

// Contract Review
export interface ContractReviewResponse {
  review: string;
  contract_type: string;
  counterparty_type: string;
  success: boolean;
}

// OPC Hub types
export interface OpcHub {
  id: string;
  country_code: string;
  city: string;
  currency: string;
  corporate_tax_rate: number;
  dividend_tax_rate: number;
  capital_gains_tax: number;
  pass_through_allowed: number;
  zero_tax_on_reinvested: number;
  foreign_income_exempt: number;
  digital_nomad_visa: number;
  col_index: number;
  avg_coworking_desk: number;
  avg_apartment_rent_1bed: number;
  setup_cost: number;
  annual_compliance_cost: number;
  hub_notes?: string;
  is_active: number;
  flag: string;
  corporate_tax_rate_label: string;
  dividend_tax_rate_label: string;
  rent_label: string;
  feature_badges: Array<{
    label: string;
    variant?: "default" | "secondary" | "destructive" | "outline";
    icon?: string;
    class_name?: string;
  }>;
}

export interface OpcCalculatorResult {
  hub_id: string;
  city: string;
  friction_costs: number;
  living_costs: number;
  net_take_home: number;
  mrr_survival_line: number;
  take_home_percentage: number;
  setup_and_maintenance_roi: number;
  tax_sandbox_type: string;
  total_tax_paid: number;
  rank: number;
  is_best: boolean;
  friction_costs_label: string;
  living_costs_label: string;
  net_take_home_label: string;
  mrr_survival_line_label: string;
  take_home_percentage_label: string;
  setup_and_maintenance_roi_label: string;
  total_tax_paid_label: string;
  tax_type_badge: {
    label: string;
    class_name?: string;
  };
}

export const globalOpcHubs = {
  list: () =>
    api.get<{ data: OpcHub[]; count: number }>("/api/global_opc_hubs"),
  calculate: (revenue: number, colIndex: number) =>
    api.post<OpcCalculatorResult[]>("/api/global_opc_hubs/calculator", {
      est_annual_revenue: revenue,
      current_base_col_index: colIndex,
    }),
};

export interface Asset {
  id: string;
  company_id: string;
  created_by?: string;
  name: string;
  asset_type?: string;
  status?: string;
  description?: string;
  serial_number?: string;
  location?: string;
  purchase_date?: string;
  purchase_price?: number;
  current_value?: number;
  expiry_date?: string;
  category_id?: string;
  image_url?: string;
  created_at?: string;
  updated_at?: string;
}

export interface CreateAssetInput {
  name: string;
  asset_type?: string;
  description?: string;
  serial_number?: string;
  location?: string;
  purchase_date?: string;
  purchase_price?: number;
  current_value?: number;
  category_id?: string;
}

export interface Invoice {
  id: string;
  company_id: string;
  created_by?: string;
  invoice_number: string;
  invoice_type?: string;
  amount: number;
  tax_amount?: number;
  status?: string;
  invoice_date?: string;
  due_date?: string;
  supplier?: string;
  description?: string;
  file_path?: string;
  file_name?: string;
  file_label?: string;
  has_file?: boolean;
  created_at?: string;
  updated_at?: string;
}

export interface CreateInvoiceInput {
  invoice_number: string;
  invoice_type?: string;
  amount: number;
  tax_amount?: number;
  invoice_date?: string;
  due_date?: string;
  supplier?: string;
  description?: string;
  file_path?: string | null;
}

export interface Employee {
  id: string;
  company_id: string;
  name: string;
  email?: string;
  phone?: string;
  department?: string;
  position?: string;
  hire_date?: string;
  status?: string;
  created_at?: string;
  updated_at?: string;
}

export interface EmployeeInput {
  name: string;
  email?: string;
  phone?: string;
  department?: string;
  position?: string;
  hire_date?: string;
  employment_type?: string;
  status?: string;
  base_salary?: number;
  social_security_base?: number;
  housing_fund_base?: number;
  notes?: string;
  avatar_url?: string;
  contract_start_date?: string;
  contract_end_date?: string;
  bank_account?: string;
  bank_name?: string;
  emergency_contact?: string;
  emergency_phone?: string;
  id_number?: string;
}

export interface Budget {
  id: string;
  company_id: string;
  category: string;
  budget_type: string;
  period: string;
  budget_amount?: number;
  actual_amount?: number;
  status?: string;
  notes?: string;
  created_at?: string;
  updated_at?: string;
}

export interface CreateBudgetInput {
  category: string;
  budget_type: string;
  period: string;
  budget_amount?: number;
  notes?: string;
}

export interface BankAccount {
  id: string;
  company_id: string;
  bank_name: string;
  account_name: string;
  account_number: string;
  account_type?: string;
  balance?: number;
  currency?: string;
  status?: string;
  notes?: string;
  created_at?: string;
  updated_at?: string;
}

export interface CreateBankAccountInput {
  bank_name: string;
  account_name: string;
  account_number: string;
  account_type?: string;
  balance?: number;
  currency?: string;
  notes?: string;
}

// Cloud Cost Management Interfaces
export interface CloudProvider {
  id: string;
  company_id: string;
  provider_name: string;
  provider_type: string;
  account_name: string;
  account_id?: string;
  region?: string;
  contact_person?: string;
  contact_email?: string;
  contact_phone?: string;
  notes?: string;
  created_at?: string;
  updated_at?: string;
}

export interface CloudResource {
  id: string;
  company_id: string;
  provider_id?: string;
  resource_type: string;
  resource_name: string;
  instance_id?: string;
  specification?: string;
  region?: string;
  purchase_type?: string;
  start_date: string;
  end_date?: string;
  monthly_cost?: number;
  status: string;
  notes?: string;
  created_at?: string;
  updated_at?: string;
}

export interface DomainSubscription {
  id: string;
  company_id: string;
  domain_name: string;
  registrar: string;
  registration_date: string;
  expiry_date: string;
  initial_purchase_cost?: number;
  annual_fee?: number;
  auto_renewal?: boolean;
  dns_provider?: string;
  status: string;
  contact_email?: string;
  notes?: string;
  created_at?: string;
  updated_at?: string;
}

export interface SoftwareSubscription {
  id: string;
  company_id: string;
  software_name: string;
  vendor: string;
  subscription_type: string;
  plan_name?: string;
  license_count?: number;
  total_price?: number;
  billing_cycle: string;
  start_date: string;
  next_billing_date: string;
  auto_renewal?: boolean;
  status: string;
  usage_department?: string;
  account_email?: string;
  notes?: string;
  created_at?: string;
  updated_at?: string;
}

// Cloud Cost Management APIs
export const cloudProviders = {
  list: () =>
    api.get<{ data: CloudProvider[]; count: number }>("/api/cloud_providers"),
  get: (id: string) => api.get<CloudProvider>(`/api/cloud_providers/${id}`),
  create: (data: Partial<CloudProvider>) =>
    api.post<CloudProvider>("/api/cloud_providers", data),
  update: (id: string, data: Partial<CloudProvider>) =>
    api.put<CloudProvider>(`/api/cloud_providers/${id}`, data),
  delete: (id: string) => api.delete(`/api/cloud_providers/${id}`),
};

export const cloudResources = {
  list: () =>
    api.get<{ data: CloudResource[]; count: number }>("/api/cloud_resources"),
  get: (id: string) => api.get<CloudResource>(`/api/cloud_resources/${id}`),
  create: (data: Partial<CloudResource>) =>
    api.post<CloudResource>("/api/cloud_resources", data),
  update: (id: string, data: Partial<CloudResource>) =>
    api.put<CloudResource>(`/api/cloud_resources/${id}`, data),
  delete: (id: string) => api.delete(`/api/cloud_resources/${id}`),
};

export const domainSubscriptions = {
  list: () =>
    api.get<{ data: DomainSubscription[]; count: number }>(
      "/api/domain_subscriptions",
    ),
  get: (id: string) =>
    api.get<DomainSubscription>(`/api/domain_subscriptions/${id}`),
  create: (data: Partial<DomainSubscription>) =>
    api.post<DomainSubscription>("/api/domain_subscriptions", data),
  update: (id: string, data: Partial<DomainSubscription>) =>
    api.put<DomainSubscription>(`/api/domain_subscriptions/${id}`, data),
  delete: (id: string) => api.delete(`/api/domain_subscriptions/${id}`),
};

export const softwareSubscriptions = {
  list: () =>
    api.get<{ data: SoftwareSubscription[]; count: number }>(
      "/api/software_subscriptions",
    ),
  get: (id: string) =>
    api.get<SoftwareSubscription>(`/api/software_subscriptions/${id}`),
  create: (data: Partial<SoftwareSubscription>) =>
    api.post<SoftwareSubscription>("/api/software_subscriptions", data),
  update: (id: string, data: Partial<SoftwareSubscription>) =>
    api.put<SoftwareSubscription>(`/api/software_subscriptions/${id}`, data),
  delete: (id: string) => api.delete(`/api/software_subscriptions/${id}`),
};

// ═══════════════════════════════════════════════════════════════
// Generic CRUD Factory — one-line SDK for any entity
// ═══════════════════════════════════════════════════════════════

/**
 * Create a typed CRUD SDK for any generic entity.
 * Usage: const projects = genericCrud<Project>('/api/v1/projects');
 */
export function genericCrud<T>(basePath: string) {
  return {
    list: (params?: Record<string, string | number>) => {
      const qs = params
        ? "?" +
          new URLSearchParams(
            Object.entries(params).map(([k, v]) => [k, String(v)]),
          ).toString()
        : "";
      return api.get<{ data: T[]; count: number }>(`${basePath}${qs}`);
    },
    get: (id: string) => api.get<T>(`${basePath}/${id}`),
    create: (data: Partial<T>) => api.post<T>(basePath, data),
    update: (id: string, data: Partial<T>) =>
      api.put<T>(`${basePath}/${id}`, data),
    delete: (id: string) => api.delete(`${basePath}/${id}`),
  };
}

// ═══════════════════════════════════════════════════════════════
// Module-Specific SDKs (per SystemBlueprint.md)
// ═══════════════════════════════════════════════════════════════

// ── Bank Transactions ──────────────────────────────────────────
export const bankTransactions = {
  list: () =>
    api.get<{ data: unknown[]; count: number }>("/api/bank/transactions"),
  create: (data: {
    account_id: string;
    transaction_type: "deposit" | "withdrawal";
    amount: number;
    transaction_date: string;
    description?: string;
  }) => api.post<unknown>("/api/bank/transactions", data),
};

// ── Reports ────────────────────────────────────────────────────
export const reports = {
  financialHealth: () => api.get<unknown>("/api/v1/reports/financial-health"),
  cashflowForecast: () => api.get<unknown>("/api/v1/reports/cashflow-forecast"),
  profitAnalysis: () => api.get<unknown>("/api/v1/reports/profit-analysis"),
  dealsConversion: () => api.get<unknown>("/api/v1/reports/deals-conversion"),
};

// ── Tax Tools ──────────────────────────────────────────────────
export const taxTools = {
  calculate: (amount?: number) =>
    api.get<unknown>(
      `/api/v1/tax/calculate${amount ? `?amount=${amount}` : ""}`,
    ),
  complianceCheck: () => api.get<unknown>("/api/v1/tax/compliance-check"),
  invoiceRequirements: () =>
    api.get<unknown>("/api/v1/tax/invoice-requirements"),
  inputCredit: () => api.get<unknown>("/api/v1/tax/input-credit"),
};

// ── Currency ───────────────────────────────────────────────────
export const currency = {
  convert: (from: string, to: string, amount: number) =>
    api.get<unknown>(
      `/api/v1/currency/convert?from=${from}&to=${to}&amount=${amount}`,
    ),
  balances: () => api.get<unknown>("/api/v1/currency/balances"),
};

// ── AR (Accounts Receivable) ───────────────────────────────────
export const ar = {
  aging: () => api.get<unknown>("/api/v1/ar/aging"),
  summary: () => api.get<unknown>("/api/v1/ar/summary"),
};

// ── Webhooks ───────────────────────────────────────────────────
export const webhooks = {
  list: () => api.get<{ data: unknown[]; count: number }>("/api/v1/webhooks"),
  create: (url: string, events: string[]) =>
    api.post<unknown>("/api/v1/webhooks", { url, events }),
  delete: (id: string) => api.delete(`/api/v1/webhooks/${id}`),
  test: (id: string) => api.post<unknown>(`/api/v1/webhooks/${id}/test`, {}),
};

// ── API Keys ───────────────────────────────────────────────────
export const apiKeys = {
  list: () => api.get<{ keys: unknown[]; total: number }>("/api/api-keys"),
  create: (name?: string, expiresInDays?: number) =>
    api.post<unknown>("/api/api-keys", {
      name,
      expires_in_days: expiresInDays,
    }),
  revoke: (id: string) => api.post<unknown>(`/api/api-keys/${id}/revoke`, {}),
  delete: (id: string) => api.delete(`/api/api-keys/${id}`),
};

// ── Conversions ────────────────────────────────────────────────
export const conversions = {
  timeToInvoice: (timeEntryIds: string[], hourlyRate?: number) =>
    api.post<unknown>("/api/v1/time_entries/invoice", {
      time_entry_ids: timeEntryIds,
      hourly_rate: hourlyRate,
    }),
  quoteToInvoice: (quoteId: string) =>
    api.post<unknown>(`/api/v1/quotes/${quoteId}/convert-to-invoice`, {}),
};

// ── PDF Generation ─────────────────────────────────────────────
export const pdf = {
  invoiceUrl: (id: string) => `${API_BASE}/api/v1/invoices/${id}/pdf`,
  quoteUrl: (id: string) => `${API_BASE}/api/v1/quotes/${id}/pdf`,
  // Fetch a server-rendered PDF (auth-gated, company-scoped) as a blob and
  // trigger a browser "save as". api.get() parses JSON, so PDFs need raw fetch.
  download: async (url: string, filename: string): Promise<void> => {
    const response = await fetch(url, {
      credentials: "include",
    });
    if (!response.ok) {
      throw new Error(`PDF download failed: ${response.status}`);
    }
    const blob = await response.blob();
    const objectUrl = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = objectUrl;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(objectUrl);
  },
  downloadInvoice(id: string, filename: string): Promise<void> {
    return pdf.download(pdf.invoiceUrl(id), filename);
  },
  downloadQuote(id: string, filename: string): Promise<void> {
    return pdf.download(pdf.quoteUrl(id), filename);
  },
};

// ── Export ──────────────────────────────────────────────────────
export const exportData = {
  csv: (table: string) => api.get<unknown>(`/api/v1/export/${table}`),
  auditExport: () => api.get<unknown>("/api/v1/audit/export"),
};

// ── System ─────────────────────────────────────────────────────
export const system = {
  status: () => api.get<unknown>("/api/v1/system/status"),
  dashboard: () => api.get<unknown>("/api/v1/stats/dashboard"),
  operationLogs: (limit = 20) =>
    api.get<{ data: unknown[] }>(`/api/v1/operation_logs?limit=${limit}`),
  errorLogs: (limit = 100) =>
    api.get<{ data: unknown[] }>(
      `/api/v1/error_logs?order_by=created_at&order_dir=desc&limit=${limit}`,
    ),
  securityAudit: () => api.get<unknown>("/api/v1/security/audit"),
  apiCatalog: () => api.get<unknown>("/api/v1/api-catalog"),
};

// ── Email ──────────────────────────────────────────────────────
export const email = {
  // Reuse operationId for retries of one intended message; a new key means a new send.
  send: (data: { to: string; subject: string; body: string }, operationId = crypto.randomUUID()) =>
    api.post<{ status: string; delivered: boolean; log_id: string }>("/api/v1/email/send", { ...data, operation_id: operationId }),
  logs: () => api.get<{ data: unknown[] }>("/api/v1/email/logs"),
};

// ── Bank Reconciliation ────────────────────────────────────────
export const reconciliation = {
  reconcile: (data: unknown) =>
    api.post<unknown>("/api/v1/bank/reconcile", data),
};

// ═══════════════════════════════════════════════════════════════
// Pre-Built Generic CRUDs (all remaining entities from Blueprint)
// ═══════════════════════════════════════════════════════════════

export const contracts = genericCrud<unknown>("/api/v1/contracts");
export const projects = genericCrud<unknown>("/api/v1/projects");
export const payments = genericCrud<unknown>("/api/v1/payments");
export const salary = genericCrud<unknown>("/api/v1/salary");
export const rent = genericCrud<unknown>("/api/v1/rent");
export const licenses = genericCrud<unknown>("/api/v1/licenses");
export const reminders = genericCrud<unknown>("/api/v1/reminders");
export const kpi = genericCrud<unknown>("/api/v1/kpi");
export const kpiConfigs = genericCrud<unknown>("/api/v1/kpi_configs");
export const kpiRecords = genericCrud<unknown>("/api/v1/kpi_records");
export const tax = genericCrud<unknown>("/api/v1/tax");
export const categories = genericCrud<unknown>("/api/v1/categories");
export const socialInsurance = genericCrud<unknown>(
  "/api/v1/social_insurance_config",
);
export const companyInfo = genericCrud<unknown>("/api/v1/company_info");
export const branches = genericCrud<unknown>("/api/v1/branches");
export const milestones = genericCrud<unknown>("/api/v1/milestones");
export const dataAssetMeta = genericCrud<unknown>("/api/v1/data_asset_meta");
export const rdCosts = genericCrud<unknown>("/api/v1/rd_costs");
export const applications = genericCrud<unknown>("/api/v1/applications");
export const applicationMaterials = genericCrud<unknown>(
  "/api/v1/application_materials",
);
export const materialLibrary = genericCrud<unknown>("/api/v1/material_library");
export const materialTemplates = genericCrud<unknown>(
  "/api/v1/material_templates",
);
export const materialVersions = genericCrud<unknown>(
  "/api/v1/material_versions",
);
export const utilities = genericCrud<unknown>("/api/v1/utilities");
export const taxDeclarations = genericCrud<unknown>("/api/v1/tax_declarations");
export const assetRecords = genericCrud<unknown>("/api/v1/asset_records");
export const profiles = genericCrud<unknown>("/api/v1/profiles");
export const userRoles = genericCrud<unknown>("/api/v1/user_roles");
export const aiConfigurations = genericCrud<unknown>(
  "/api/v1/ai_configurations",
);
export const emailConfigurations = genericCrud<unknown>(
  "/api/v1/email_configurations",
);
export const vendors = genericCrud<unknown>("/api/v1/vendors");
export const exchangeRates = genericCrud<unknown>("/api/v1/exchange_rates");
export const clients = genericCrud<unknown>("/api/v1/clients");
export const contacts = genericCrud<unknown>("/api/v1/contacts");
export const interactions = genericCrud<unknown>("/api/v1/interactions");
export const deals = genericCrud<unknown>("/api/v1/deals");
export const quotes = genericCrud<unknown>("/api/v1/quotes");
export const quoteItems = genericCrud<unknown>("/api/v1/quote_items");
export const timeEntries = genericCrud<unknown>("/api/v1/time_entries");
export const recurringInvoices = genericCrud<unknown>(
  "/api/v1/recurring_invoices",
);
export const invoiceReminders = genericCrud<unknown>(
  "/api/v1/invoice_reminders",
);
export const insurancePolicies = genericCrud<unknown>(
  "/api/v1/insurance_policies",
);
export const pensionPlans = genericCrud<unknown>("/api/v1/pension_plans");
export const taxJurisdictions = genericCrud<unknown>(
  "/api/v1/tax_jurisdictions",
);
export const taxDeadlines = genericCrud<unknown>("/api/v1/tax_deadlines");
export const cloudBills = genericCrud<unknown>("/api/v1/cloud_bills");
export const errorLogs = genericCrud<unknown>("/api/v1/error_logs");
