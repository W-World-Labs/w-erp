import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";

export const WORKBENCH_SUMMARY_ENDPOINT = "/api/v1/workbench/summary";

type Priority = "critical" | "high" | "medium" | "low";
type UrgentType = "invoice" | "contract" | "asset" | "tax" | "application" | "salary";

interface BackendBadge {
  label: string;
  class_name: string;
  color_class: string;
}

interface BackendUrgentItem {
  id: string;
  title: string;
  type: UrgentType;
  priority: Priority;
  due_date?: string | null;
  days_left?: number | null;
  days_left_label?: string | null;
  priority_badge: BackendBadge;
  type_icon: string;
  description: string;
  action: string;
  path: string;
}

interface BackendTodoItem {
  id: string;
  title: string;
  category: string;
  priority: Priority;
  completed: boolean;
  path: string;
}

interface BackendHeaderBadge {
  label: string;
  variant: "default" | "secondary" | "destructive" | "outline";
  class_name?: string;
}

interface BackendOperatingAction {
  route?: string;
  label?: string;
  description?: string;
  action?: unknown | null;
}

interface BackendOperatingControlItem {
  id: string;
  type: string;
  title: string;
  description: string;
  priority: Priority;
  route: string;
  action_label: string;
  badge: BackendBadge;
  metadata?: unknown;
}

interface BackendOperatingControl {
  id?: string;
  title?: string;
  status?: string;
  status_label?: string;
  tone?: string;
  backbone_rate?: number;
  backbone_rate_label?: string;
  completed?: boolean;
  blocked_step_id?: string | null;
  blocked_step_label?: string | null;
  risk_count?: number;
  critical_count?: number;
  risk_count_label?: string;
  summary_label?: string;
  next_action?: BackendOperatingAction | null;
  control_items?: BackendOperatingControlItem[];
}

interface BackendWorkbenchSummary {
  greeting?: string;
  generated_date_label?: string;
  operating_control?: BackendOperatingControl | null;
  stats?: {
    pending_invoices?: number;
    pending_invoices_label?: string;
    active_contracts?: number;
    active_contracts_label?: string;
    expiring_assets?: number;
    expiring_assets_label?: string;
    ongoing_projects?: number;
    ongoing_projects_label?: string;
    employees?: number;
    employees_label?: string;
    monthly_expense?: number;
    monthly_expense_label?: string;
  };
  header_badges?: BackendHeaderBadge[];
  urgent_items?: BackendUrgentItem[];
  application_reminders?: BackendUrgentItem[];
  asset_expiry_reminders?: BackendUrgentItem[];
  attention_items?: BackendUrgentItem[];
  today_tasks?: unknown[];
  today_tasks_count?: number;
  todos?: BackendTodoItem[];
  ai_suggestions?: string[];
  completion_rate?: number;
  stage_stats?: {
    total_assets?: number;
    total_assets_label?: string;
    total_value?: number;
    total_value_label?: string;
    pending_invoices?: number;
    pending_invoices_label?: string;
    total_salary?: number;
    total_salary_label?: string;
    ongoing_projects?: number;
    ongoing_projects_label?: string;
    active_contracts?: number;
    active_contracts_label?: string;
    employee_count?: number;
    employee_count_label?: string;
    total_projects?: number;
    total_projects_label?: string;
    total_contracts?: number;
    total_contracts_label?: string;
    project_execution_rate_label?: string;
    contract_activity_rate_label?: string;
  };
  monthly_expense_trend?: Array<{
    month: string;
    salary: number;
    salary_label?: string;
    invoice: number;
    invoice_label?: string;
    total: number;
    total_label?: string;
  }>;
  overview_cards?: BackendOverviewCard[];
  quick_actions?: BackendQuickAction[];
  survival_items?: BackendUrgentItem[];
  survival_count?: number;
  survival_count_label?: string;
  survival_title?: string;
  survival_subtitle?: string;
  guidance?: BackendGuidance;
}

interface BackendGuidanceStep {
  id: string;
  label: string;
  description: string;
  completed: boolean;
  route: string;
}

interface BackendGuidance {
  visible: boolean;
  kind: string;
  title: string;
  subtitle: string;
  progress_label: string;
  next_label: string;
  next_description: string;
  next_route: string;
  empty_urgent_title: string;
  empty_urgent_description: string;
  steps?: BackendGuidanceStep[];
}

interface BackendOverviewCard {
  key: string;
  icon: string;
  title: string;
  value_label: string;
  subtitle?: string | null;
  trend_label?: string | null;
  trend_up?: boolean | null;
  trend_variant?: "default" | "secondary" | null;
  color: "blue" | "green" | "purple" | "orange";
  class_name: string;
}

interface BackendQuickAction {
  icon: string;
  label: string;
  description: string;
  path: string;
  color: string;
}

export interface UrgentItem {
  id: string;
  title: string;
  type: UrgentType;
  priority: Priority;
  dueDate?: string;
  daysLeft?: number;
  daysLeftLabel?: string;
  priorityBadge: BackendBadge;
  typeIcon: string;
  description: string;
  action: string;
  path: string;
}

export interface WorkbenchOverviewCard {
  key: string;
  icon: string;
  title: string;
  valueLabel: string;
  subtitle?: string | null;
  trendLabel?: string | null;
  trendUp?: boolean | null;
  trendVariant?: "default" | "secondary" | null;
  color: "blue" | "green" | "purple" | "orange";
  className: string;
}

export interface WorkbenchQuickAction {
  icon: string;
  label: string;
  description: string;
  path: string;
  color: string;
}

export interface WorkbenchHeaderBadge {
  label: string;
  variant: "default" | "secondary" | "destructive" | "outline";
  className?: string;
}

export interface OperatingControlAction {
  route: string;
  label: string;
  description: string;
  action?: unknown | null;
}

export interface OperatingControlItem {
  id: string;
  type: string;
  title: string;
  description: string;
  priority: Priority;
  route: string;
  actionLabel: string;
  badge: BackendBadge;
  metadata?: unknown;
}

export interface OperatingControlView {
  id: string;
  title: string;
  status: string;
  statusLabel: string;
  tone: string;
  backboneRate: number;
  backboneRateLabel: string;
  completed: boolean;
  blockedStepId?: string | null;
  blockedStepLabel?: string | null;
  riskCount: number;
  criticalCount: number;
  riskCountLabel: string;
  summaryLabel: string;
  nextAction: OperatingControlAction;
  controlItems: OperatingControlItem[];
}

export interface WorkbenchSummaryView {
  generatedDateLabel: string;
  operatingControl: OperatingControlView;
  urgentItems: UrgentItem[];
  applicationReminders: UrgentItem[];
  assetExpiryReminders: UrgentItem[];
  attentionItems: UrgentItem[];
  stats: {
    pendingInvoices: number;
    pendingInvoicesLabel: string;
    activeContracts: number;
    activeContractsLabel: string;
    expiringAssets: number;
    expiringAssetsLabel: string;
    ongoingProjects: number;
    ongoingProjectsLabel: string;
    totalEmployees: number;
    totalEmployeesLabel: string;
    monthlyExpense: number;
    monthlyExpenseLabel: string;
  };
  headerBadges: WorkbenchHeaderBadge[];
  greeting: string;
  todayTasks: number;
  todos: BackendTodoItem[];
  aiSuggestions: string[];
  completionRate: number;
  stageStats: {
    totalAssets: number;
    totalAssetsLabel: string;
    totalValue: number;
    totalValueLabel: string;
    pendingInvoices: number;
    pendingInvoicesLabel: string;
    totalSalary: number;
    totalSalaryLabel: string;
    ongoingProjects: number;
    ongoingProjectsLabel: string;
    activeContracts: number;
    activeContractsLabel: string;
    employeeCount: number;
    employeeCountLabel: string;
    totalProjects: number;
    totalProjectsLabel: string;
    totalContracts: number;
    totalContractsLabel: string;
    projectExecutionRateLabel: string;
    contractActivityRateLabel: string;
  };
  monthlyExpenseTrend: Array<{
    month: string;
    salary: number;
    salaryLabel?: string;
    invoice: number;
    invoiceLabel?: string;
    total: number;
    totalLabel?: string;
  }>;
  overviewCards: WorkbenchOverviewCard[];
  quickActions: WorkbenchQuickAction[];
  survivalItems: UrgentItem[];
  survivalCount: number;
  survivalCountLabel: string;
  survivalTitle: string;
  survivalSubtitle: string;
  guidance: WorkbenchGuidanceView;
}

export interface WorkbenchGuidanceStepView {
  id: string;
  label: string;
  description: string;
  completed: boolean;
  route: string;
}

export interface WorkbenchGuidanceView {
  visible: boolean;
  kind: string;
  title: string;
  subtitle: string;
  progressLabel: string;
  nextLabel: string;
  nextDescription: string;
  nextRoute: string;
  emptyUrgentTitle: string;
  emptyUrgentDescription: string;
  steps: WorkbenchGuidanceStepView[];
}

const emptyOperatingControl: OperatingControlView = {
  id: "operating_control",
  title: "运营指挥台",
  status: "healthy",
  statusLabel: "健康",
  tone: "success",
  backboneRate: 0,
  backboneRateLabel: "0%",
  completed: false,
  blockedStepId: null,
  blockedStepLabel: null,
  riskCount: 0,
  criticalCount: 0,
  riskCountLabel: "0 个风险",
  summaryLabel: "",
  nextAction: {
    route: "/dashboard",
    label: "查看仪表盘",
    description: "",
    action: null,
  },
  controlItems: [],
};

const emptyWorkbench: WorkbenchSummaryView = {
  generatedDateLabel: "",
  operatingControl: emptyOperatingControl,
  urgentItems: [],
  applicationReminders: [],
  assetExpiryReminders: [],
  attentionItems: [],
  stats: {
    pendingInvoices: 0,
    pendingInvoicesLabel: "0 张待审发票",
    activeContracts: 0,
    activeContractsLabel: "0",
    expiringAssets: 0,
    expiringAssetsLabel: "0 项即将到期",
    ongoingProjects: 0,
    ongoingProjectsLabel: "0",
    totalEmployees: 0,
    totalEmployeesLabel: "0",
    monthlyExpense: 0,
    monthlyExpenseLabel: "¥0.00",
  },
  headerBadges: [],
  greeting: "",
  todayTasks: 0,
  todos: [],
  aiSuggestions: [],
  completionRate: 0,
  stageStats: {
    totalAssets: 0,
    totalAssetsLabel: "0",
    totalValue: 0,
    totalValueLabel: "¥0.00",
    pendingInvoices: 0,
    pendingInvoicesLabel: "0",
    totalSalary: 0,
    totalSalaryLabel: "¥0.00",
    ongoingProjects: 0,
    ongoingProjectsLabel: "0",
    activeContracts: 0,
    activeContractsLabel: "0",
    employeeCount: 0,
    employeeCountLabel: "0",
    totalProjects: 0,
    totalProjectsLabel: "0",
    totalContracts: 0,
    totalContractsLabel: "0",
    projectExecutionRateLabel: "0%",
    contractActivityRateLabel: "0%",
  },
  monthlyExpenseTrend: [],
  overviewCards: [],
  quickActions: [],
  survivalItems: [],
  survivalCount: 0,
  survivalCountLabel: "",
  survivalTitle: "",
  survivalSubtitle: "",
  guidance: {
    visible: false,
    kind: "",
    title: "",
    subtitle: "",
    progressLabel: "",
    nextLabel: "",
    nextDescription: "",
    nextRoute: "/dashboard",
    emptyUrgentTitle: "",
    emptyUrgentDescription: "",
    steps: [],
  },
};

function normalizeUrgentItem(item: BackendUrgentItem): UrgentItem {
  return {
    id: item.id,
    title: item.title,
    type: item.type,
    priority: item.priority,
    dueDate: item.due_date || undefined,
    daysLeft: item.days_left ?? undefined,
    daysLeftLabel: item.days_left_label || undefined,
    priorityBadge: item.priority_badge,
    typeIcon: item.type_icon,
    description: item.description,
    action: item.action,
    path: item.path,
  };
}

function normalizeOperatingControl(
  control?: BackendOperatingControl | null
): OperatingControlView {
  if (!control) {
    return emptyOperatingControl;
  }

  const nextAction = control.next_action || {};

  return {
    id: control.id || "operating_control",
    title: control.title || "运营指挥台",
    status: control.status || "healthy",
    statusLabel: control.status_label || "健康",
    tone: control.tone || "success",
    backboneRate: control.backbone_rate || 0,
    backboneRateLabel: control.backbone_rate_label || "0%",
    completed: control.completed || false,
    blockedStepId: control.blocked_step_id || null,
    blockedStepLabel: control.blocked_step_label || null,
    riskCount: control.risk_count || 0,
    criticalCount: control.critical_count || 0,
    riskCountLabel: control.risk_count_label || "0 个风险",
    summaryLabel: control.summary_label || "",
    nextAction: {
      route: nextAction.route || "/dashboard",
      label: nextAction.label || "查看仪表盘",
      description: nextAction.description || "",
      action: nextAction.action,
    },
    controlItems: (control.control_items || []).map((item) => ({
      id: item.id,
      type: item.type,
      title: item.title,
      description: item.description,
      priority: item.priority,
      route: item.route,
      actionLabel: item.action_label,
      badge: item.badge,
      metadata: item.metadata,
    })),
  };
}

export function normalizeWorkbenchSummary(
  data?: BackendWorkbenchSummary | null
): WorkbenchSummaryView {
  if (!data) {
    return emptyWorkbench;
  }

  return {
    generatedDateLabel: data.generated_date_label || "",
    operatingControl: normalizeOperatingControl(data.operating_control),
    urgentItems: (data.urgent_items || []).map(normalizeUrgentItem),
    applicationReminders: (data.application_reminders || []).map(normalizeUrgentItem),
    assetExpiryReminders: (data.asset_expiry_reminders || []).map(normalizeUrgentItem),
    attentionItems: (data.attention_items || []).map(normalizeUrgentItem),
    stats: {
      pendingInvoices: data.stats?.pending_invoices || 0,
      pendingInvoicesLabel: data.stats?.pending_invoices_label || "0 张待审发票",
      activeContracts: data.stats?.active_contracts || 0,
      activeContractsLabel: data.stats?.active_contracts_label || "0",
      expiringAssets: data.stats?.expiring_assets || 0,
      expiringAssetsLabel: data.stats?.expiring_assets_label || "0 项即将到期",
      ongoingProjects: data.stats?.ongoing_projects || 0,
      ongoingProjectsLabel: data.stats?.ongoing_projects_label || "0",
      totalEmployees: data.stats?.employees || 0,
      totalEmployeesLabel: data.stats?.employees_label || "0",
      monthlyExpense: data.stats?.monthly_expense || 0,
      monthlyExpenseLabel: data.stats?.monthly_expense_label || "¥0.00",
    },
    headerBadges: (data.header_badges || []).map((badge) => ({
      label: badge.label,
      variant: badge.variant,
      className: badge.class_name,
    })),
    greeting: data.greeting || "",
    todayTasks: data.today_tasks_count || 0,
    todos: data.todos || [],
    aiSuggestions: data.ai_suggestions || [],
    completionRate: data.completion_rate || 0,
    stageStats: {
      totalAssets: data.stage_stats?.total_assets || 0,
      totalAssetsLabel: data.stage_stats?.total_assets_label || "0",
      totalValue: data.stage_stats?.total_value || 0,
      totalValueLabel: data.stage_stats?.total_value_label || "¥0.00",
      pendingInvoices: data.stage_stats?.pending_invoices || 0,
      pendingInvoicesLabel: data.stage_stats?.pending_invoices_label || "0",
      totalSalary: data.stage_stats?.total_salary || 0,
      totalSalaryLabel: data.stage_stats?.total_salary_label || "¥0.00",
      ongoingProjects: data.stage_stats?.ongoing_projects || 0,
      ongoingProjectsLabel: data.stage_stats?.ongoing_projects_label || "0",
      activeContracts: data.stage_stats?.active_contracts || 0,
      activeContractsLabel: data.stage_stats?.active_contracts_label || "0",
      employeeCount: data.stage_stats?.employee_count || 0,
      employeeCountLabel: data.stage_stats?.employee_count_label || "0",
      totalProjects: data.stage_stats?.total_projects || 0,
      totalProjectsLabel: data.stage_stats?.total_projects_label || "0",
      totalContracts: data.stage_stats?.total_contracts || 0,
      totalContractsLabel: data.stage_stats?.total_contracts_label || "0",
      projectExecutionRateLabel: data.stage_stats?.project_execution_rate_label || "0%",
      contractActivityRateLabel: data.stage_stats?.contract_activity_rate_label || "0%",
    },
    monthlyExpenseTrend: (data.monthly_expense_trend || []).map((point) => ({
      month: point.month,
      salary: point.salary,
      salaryLabel: point.salary_label,
      invoice: point.invoice,
      invoiceLabel: point.invoice_label,
      total: point.total,
      totalLabel: point.total_label,
    })),
    overviewCards: (data.overview_cards || []).map((card) => ({
      key: card.key,
      icon: card.icon,
      title: card.title,
      valueLabel: card.value_label,
      subtitle: card.subtitle,
      trendLabel: card.trend_label,
      trendUp: card.trend_up,
      trendVariant: card.trend_variant,
      color: card.color,
      className: card.class_name,
    })),
    quickActions: data.quick_actions || [],
    survivalItems: (data.survival_items || []).map(normalizeUrgentItem),
    survivalCount: data.survival_count || 0,
    survivalCountLabel: data.survival_count_label || "",
    survivalTitle: data.survival_title || "",
    survivalSubtitle: data.survival_subtitle || "",
    guidance: {
      visible: data.guidance?.visible || false,
      kind: data.guidance?.kind || "",
      title: data.guidance?.title || "",
      subtitle: data.guidance?.subtitle || "",
      progressLabel: data.guidance?.progress_label || "",
      nextLabel: data.guidance?.next_label || "",
      nextDescription: data.guidance?.next_description || "",
      nextRoute: data.guidance?.next_route || "/dashboard",
      emptyUrgentTitle: data.guidance?.empty_urgent_title || "",
      emptyUrgentDescription: data.guidance?.empty_urgent_description || "",
      steps: (data.guidance?.steps || []).map((step) => ({
        id: step.id,
        label: step.label,
        description: step.description,
        completed: step.completed,
        route: step.route,
      })),
    },
  };
}

export function useSmartWorkbench() {
  const { data, isLoading, isError, refetch } = useQuery({
    queryKey: ["workbench-summary"],
    queryFn: async () => api.get<BackendWorkbenchSummary>(WORKBENCH_SUMMARY_ENDPOINT),
    staleTime: 60 * 1000,
  });

  return {
    ...normalizeWorkbenchSummary(data),
    isLoading,
    isError,
    refetch,
  };
}
