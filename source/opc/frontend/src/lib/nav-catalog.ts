/** Default sidebar catalog. Item `id` is stable; prefs hide/reorder by id. */

export type NavIconName =
  | "layout-dashboard"
  | "trending-up"
  | "building"
  | "brain"
  | "file-text"
  | "credit-card"
  | "landmark"
  | "scale"
  | "banknote"
  | "home"
  | "zap"
  | "calculator"
  | "repeat"
  | "calendar-range"
  | "package"
  | "rocket"
  | "globe-2"
  | "file-check"
  | "building-2"
  | "id-card"
  | "handshake"
  | "target"
  | "clipboard-list"
  | "timer"
  | "shield-check"
  | "users"
  | "database"
  | "bell"
  | "user-circle"
  | "settings"
  | "scroll-text"
  | "dollar-sign"
  | "briefcase";

export type NavItemDef = {
  id: string;
  icon: NavIconName;
  label: string;
  path: string;
  showBadge?: boolean;
  adminOnly?: boolean;
  /** Cannot be hidden; keeps dashboard and the settings editor reachable. */
  locked?: boolean;
};

export type NavCategoryDef = {
  id: string;
  label: string;
  icon?: NavIconName;
  items: NavItemDef[];
};

export const MENU_CATEGORIES: NavCategoryDef[] = [
  {
    id: "overview",
    label: "概览",
    items: [
      {
        id: "dashboard",
        icon: "layout-dashboard",
        label: "仪表盘",
        path: "/",
        locked: true,
      },
      {
        id: "kpi-dashboard",
        icon: "trending-up",
        label: "经营数据看板",
        path: "/kpi-dashboard",
      },
      {
        id: "company-info",
        icon: "building",
        label: "公司信息",
        path: "/company-info",
      },
      {
        id: "audit-center",
        icon: "brain",
        label: "AI审计中心",
        path: "/audit-center",
      },
    ],
  },
  {
    id: "finance",
    label: "财务管理",
    icon: "dollar-sign",
    items: [
      { id: "invoices", icon: "file-text", label: "发票管理", path: "/invoices" },
      {
        id: "payments",
        icon: "credit-card",
        label: "支付管理",
        path: "/payments",
      },
      {
        id: "bank-accounts",
        icon: "landmark",
        label: "银行账户",
        path: "/bank-accounts",
      },
      {
        id: "reconciliation",
        icon: "scale",
        label: "对账中心",
        path: "/reconciliation",
      },
      {
        id: "salary",
        icon: "banknote",
        label: "工资管理",
        path: "/salary",
        adminOnly: true,
      },
      { id: "rent", icon: "home", label: "场地水电", path: "/rent" },
      {
        id: "financial-reports",
        icon: "zap",
        label: "财务报表",
        path: "/financial-reports",
      },
      {
        id: "tax-management",
        icon: "calculator",
        label: "税务管理",
        path: "/tax-management",
      },
      {
        id: "budget-management",
        icon: "zap",
        label: "预算管理",
        path: "/budget-management",
      },
      {
        id: "cash-flow-forecast",
        icon: "trending-up",
        label: "现金流预测",
        path: "/cash-flow-forecast",
      },
      {
        id: "cloud-cost-management",
        icon: "credit-card",
        label: "云成本管理",
        path: "/cloud-cost-management",
      },
      {
        id: "recurring-invoices",
        icon: "repeat",
        label: "周期发票 & 催款",
        path: "/recurring-invoices",
      },
      {
        id: "tax-calendar",
        icon: "calendar-range",
        label: "全球税务日历",
        path: "/tax-calendar",
      },
    ],
  },
  {
    id: "business",
    label: "业务管理",
    icon: "briefcase",
    items: [
      { id: "assets", icon: "package", label: "资产管理", path: "/assets" },
      { id: "projects", icon: "rocket", label: "项目管理", path: "/projects" },
      { id: "policies", icon: "zap", label: "全球政策大厅", path: "/policies" },
      {
        id: "global-opc",
        icon: "globe-2",
        label: "OPC 出海沙盘",
        path: "/global-opc",
      },
      {
        id: "applications",
        icon: "file-text",
        label: "项目申报",
        path: "/applications",
      },
      {
        id: "contracts",
        icon: "file-check",
        label: "合同管理",
        path: "/contracts",
      },
      { id: "vendors", icon: "building-2", label: "友商管理", path: "/vendors" },
      { id: "licenses", icon: "id-card", label: "证照管理", path: "/licenses" },
      { id: "crm", icon: "handshake", label: "客户管理 (CRM)", path: "/crm" },
      { id: "deals", icon: "target", label: "销售机会", path: "/deals" },
      { id: "quotes", icon: "clipboard-list", label: "报价单", path: "/quotes" },
      {
        id: "time-tracking",
        icon: "timer",
        label: "时间追踪",
        path: "/time-tracking",
      },
      {
        id: "compliance",
        icon: "shield-check",
        label: "合规管理",
        path: "/compliance",
      },
    ],
  },
  {
    id: "team",
    label: "团队与档案",
    icon: "users",
    items: [
      { id: "employees", icon: "users", label: "员工管理", path: "/employees" },
      { id: "records", icon: "database", label: "数据档案", path: "/records" },
      {
        id: "reminders",
        icon: "bell",
        label: "提醒中心",
        path: "/reminders",
        showBadge: true,
      },
    ],
  },
  {
    id: "system",
    label: "系统",
    items: [
      {
        id: "profile",
        icon: "user-circle",
        label: "个人资料",
        path: "/profile",
      },
      {
        id: "settings",
        icon: "settings",
        label: "系统设置",
        path: "/settings",
        locked: true,
      },
      {
        id: "user-management",
        icon: "users",
        label: "用户管理",
        path: "/user-management",
        adminOnly: true,
      },
      {
        id: "operation-logs",
        icon: "scroll-text",
        label: "操作日志",
        path: "/operation-logs",
        adminOnly: true,
      },
    ],
  },
];

export const LOCKED_NAV_IDS = MENU_CATEGORIES.flatMap((category) =>
  category.items.filter((item) => item.locked).map((item) => item.id),
);

export function catalogItemCount(): number {
  return MENU_CATEGORIES.reduce((sum, category) => sum + category.items.length, 0);
}

export function allCatalogItems(): NavItemDef[] {
  return MENU_CATEGORIES.flatMap((category) => category.items);
}

export function catalogItemById(id: string): NavItemDef | undefined {
  return allCatalogItems().find((item) => item.id === id);
}

export function catalogCategoryById(id: string): NavCategoryDef | undefined {
  return MENU_CATEGORIES.find((category) => category.id === id);
}

export function categoryIdForItem(itemId: string): string | undefined {
  return MENU_CATEGORIES.find((category) =>
    category.items.some((item) => item.id === itemId),
  )?.id;
}

export function categoryIdForPath(path: string): string | undefined {
  return MENU_CATEGORIES.find((category) =>
    category.items.some((item) => item.path === path),
  )?.id;
}
