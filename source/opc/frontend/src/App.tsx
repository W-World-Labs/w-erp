import { Toaster } from "@/components/ui/toaster";
import { Toaster as Sonner } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import {
  MutationCache,
  QueryCache,
  QueryClient,
  QueryClientProvider,
} from "@tanstack/react-query";
import { toast } from "sonner";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { ProtectedRoute } from "./components/ProtectedRoute";
import { ErrorBoundary } from "./components/common/ErrorBoundary";
import { NetworkStatusProvider, NetworkStatusBar } from "./components/common/NetworkStatus";
import { ThemeProvider } from "./components/ThemeProvider";
import { apiErrorMessage, toastOnce } from "@/lib/apiError";
import Index from "./pages/Index";
import Auth from "./pages/Auth";

import Dashboard from "./pages/Dashboard";
import KPIDashboard from "./pages/KPIDashboard";
import Assets from "./pages/Assets";
import Invoices from "./pages/Invoices";
import Records from "./pages/Records";
import Salary from "./pages/Salary";
import Rent from "./pages/Rent";
import Contracts from "./pages/Contracts";
import Projects from "./pages/Projects";
import Applications from "./pages/Applications";
import FinancialReports from "./pages/FinancialReports";
import TaxManagement from "./pages/TaxManagement";
import BudgetManagement from "./pages/BudgetManagement";
import CashFlowForecast from "./pages/CashFlowForecast";
import CloudCostManagement from "./pages/CloudCostManagement";
import Profile from "./pages/Profile";

import UserManagement from "./pages/UserManagement";
import Settings from "./pages/Settings";
import NotFound from "./pages/NotFound";
import Licenses from "./pages/Licenses";
import Employees from "./pages/Employees";
import Reminders from "./pages/Reminders";
import CompanyInfo from "./pages/CompanyInfo";
import AuditCenter from "./pages/AuditCenter";
import Payments from "./pages/Payments";
import OperationLogs from "./pages/OperationLogs";
import ReconciliationCenter from "./pages/ReconciliationCenter";
import BankAccounts from "./pages/BankAccounts";
import Vendors from "./pages/Vendors";
import PolicyDashboard from "./pages/PolicyDashboard";
import PolicyDetail from "./pages/PolicyDetail";
import GlobalOpcHub from "./pages/GlobalOpcHub";
import ClientManagement from "./pages/ClientManagement";
import DealsManagement from "./pages/DealsManagement";
import QuotesManagement from "./pages/QuotesManagement";
import TimeTracking from "./pages/TimeTracking";
import RecurringInvoices from "./pages/RecurringInvoices";
import ComplianceManagement from "./pages/ComplianceManagement";
import TaxCalendar from "./pages/TaxCalendar";

// 配置 React Query 客户端 — 全局 Query/Mutation 错误兜底
const queryClient = new QueryClient({
  queryCache: new QueryCache({
    onError: (err, query) => {
      // 已有缓存数据的后台刷新失败不打扰用户
      if (query.state.data !== undefined) return;
      toastOnce(`数据加载失败：${apiErrorMessage(err)}`);
    },
  }),
  mutationCache: new MutationCache({
    onError: (err, _vars, _ctx, mutation) => {
      // 页面自己写了 onError 的不重复弹
      if (mutation.options.onError) return;
      toast.error(`操作失败：${apiErrorMessage(err)}`);
    },
  }),
  defaultOptions: {
    queries: {
      // 数据在 5 分钟内被认为是新鲜的
      staleTime: 1000 * 60 * 5,
      // 缓存数据保留 30 分钟
      gcTime: 1000 * 60 * 30,
      // 失败时重试 1 次
      retry: 1,
      // 窗口聚焦时不自动重新获取
      refetchOnWindowFocus: false,
      // 网络恢复时重新获取
      refetchOnReconnect: true,
    },
    mutations: {
      // mutation 不重试
      retry: 0,
    },
  },
});

const App = () => (
  <ThemeProvider>
    <QueryClientProvider client={queryClient}>
      <ErrorBoundary>
        <NetworkStatusProvider>
          <TooltipProvider>
            {/* 网络状态提示条 */}
            <NetworkStatusBar />
            <Toaster />
            <Sonner />
            <BrowserRouter>
              <Routes>
                <Route path="/auth" element={<Auth />} />
                <Route path="/" element={<ProtectedRoute><Index /></ProtectedRoute>}>
                  <Route index element={<Dashboard />} />
                  <Route path="kpi-dashboard" element={<KPIDashboard />} />
                  <Route path="company-info" element={<CompanyInfo />} />
                  <Route path="branches" element={<CompanyInfo initialTab="branches" />} />
                  <Route path="audit-center" element={<AuditCenter />} />
                  <Route path="assets" element={<Assets />} />
                  <Route path="invoices" element={<Invoices />} />
                  <Route path="records" element={<Records />} />
                  <Route path="salary" element={<ProtectedRoute requireAdmin><Salary /></ProtectedRoute>} />
                  <Route path="rent" element={<Rent />} />
                  <Route path="payments" element={<Payments />} />
                  <Route path="contracts" element={<Contracts />} />
                  <Route path="vendors" element={<Vendors />} />
                  <Route path="crm" element={<ClientManagement />} />
                  <Route path="deals" element={<DealsManagement />} />
                  <Route path="quotes" element={<QuotesManagement />} />
                  <Route path="time-tracking" element={<TimeTracking />} />
                  <Route path="recurring-invoices" element={<RecurringInvoices />} />
                  <Route path="compliance" element={<ComplianceManagement />} />
                  <Route path="tax-calendar" element={<TaxCalendar />} />
                  <Route path="projects" element={<Projects />} />
                  <Route path="applications" element={<Applications />} />
                  <Route path="policies" element={<PolicyDashboard />} />
                  <Route path="policies/:id" element={<PolicyDetail />} />
                  <Route path="global-opc" element={<GlobalOpcHub />} />
                  <Route path="financial-reports" element={<FinancialReports />} />
                  <Route path="tax-management" element={<TaxManagement />} />
                  <Route path="budget-management" element={<BudgetManagement />} />
                  <Route path="cash-flow-forecast" element={<CashFlowForecast />} />
                  <Route path="cloud-cost-management" element={<CloudCostManagement />} />
                  <Route path="licenses" element={<Licenses />} />
                  <Route path="employees" element={<Employees />} />
                  <Route path="reminders" element={<Reminders />} />
                  <Route path="user-management" element={<ProtectedRoute requireAdmin><UserManagement /></ProtectedRoute>} />
                  <Route path="operation-logs" element={<ProtectedRoute requireAdmin><OperationLogs /></ProtectedRoute>} />
                  <Route path="reconciliation" element={<ReconciliationCenter />} />
                  <Route path="bank-accounts" element={<BankAccounts />} />
                  <Route path="profile" element={<Profile />} />
                  <Route path="settings" element={<ProtectedRoute requireAdmin><Settings /></ProtectedRoute>} />
                </Route>
                {/* ADD ALL CUSTOM ROUTES ABOVE THE CATCH-ALL "*" ROUTE */}
                <Route path="*" element={<NotFound />} />
              </Routes>
            </BrowserRouter>
          </TooltipProvider>
        </NetworkStatusProvider>
      </ErrorBoundary>
    </QueryClientProvider>
  </ThemeProvider>
);

export default App;
