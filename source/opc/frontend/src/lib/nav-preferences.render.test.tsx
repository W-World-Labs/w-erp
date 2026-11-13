import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import { NavigationPreferencesCard } from "@/components/settings/NavigationPreferencesCard";
import Sidebar from "@/components/Sidebar";
import { AuthProvider } from "@/contexts/AuthContext";
import { ThemeProvider } from "@/components/ThemeProvider";
import {
  applyNavPreferences,
  emptyNavPreferences,
  toggleNavItemHidden,
} from "@/lib/nav-preferences";

const apiGet = vi.fn();
const apiPut = vi.fn();

vi.mock("@/integrations/api/client", () => ({
  api: {
    get: (...args: unknown[]) => apiGet(...args),
    put: (...args: unknown[]) => apiPut(...args),
  },
  auth: {
    getCurrentUser: vi.fn(async () => ({
      id: "user-1",
      email: "owner@opc.test",
    })),
    login: vi.fn(),
    logout: vi.fn(),
  },
  companies: {
    list: vi.fn(async () => ({ data: [] })),
    switch: vi.fn(),
    create: vi.fn(),
  },
  setCurrentCompany: vi.fn(),
  getCurrentCompanyId: vi.fn(() => null),
}));

function renderWithApp(ui: React.ReactNode, path = "/settings") {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });
  return render(
    <AuthProvider>
      <ThemeProvider>
        <QueryClientProvider client={client}>
          <MemoryRouter initialEntries={[path]}>{ui}</MemoryRouter>
        </QueryClientProvider>
      </ThemeProvider>
    </AuthProvider>,
  );
}

describe("NavigationPreferencesCard", () => {
  beforeEach(() => {
    apiGet.mockReset();
    apiPut.mockReset();
    apiGet.mockImplementation(async (url: string) => {
      if (String(url).includes("/settings/nav")) {
        return emptyNavPreferences();
      }
      if (String(url).includes("reminders")) {
        return { stats: { pending: 0 } };
      }
      return {};
    });
    apiPut.mockImplementation(async (_url: string, body: unknown) => body);
  });

  it("lists catalog items and persists hide + restore", async () => {
    const user = userEvent.setup();
    renderWithApp(<NavigationPreferencesCard />);

    expect(await screen.findByText("导航栏")).toBeInTheDocument();
    expect(screen.getByText("发票管理")).toBeInTheDocument();
    expect(screen.getByLabelText("显示 仪表盘")).toBeDisabled();
    expect(screen.getByLabelText("显示 系统设置")).toBeDisabled();

    await user.click(screen.getByLabelText("显示 发票管理"));
    await waitFor(() => expect(apiPut).toHaveBeenCalled());
    const hiddenBody = apiPut.mock.calls.at(-1)?.[1] as { hidden_ids: string[] };
    expect(hiddenBody.hidden_ids).toContain("invoices");

    await user.click(screen.getByRole("button", { name: "恢复默认" }));
    await waitFor(() => {
      const resetBody = apiPut.mock.calls.at(-1)?.[1] as { hidden_ids: string[] };
      expect(resetBody.hidden_ids).toEqual([]);
    });
  });
});

describe("Sidebar with user nav preferences", () => {
  beforeEach(() => {
    apiGet.mockReset();
    apiPut.mockReset();
    apiGet.mockImplementation(async (url: string) => {
      if (String(url).includes("/settings/nav")) {
        return toggleNavItemHidden(emptyNavPreferences(), "invoices", true);
      }
      if (String(url).includes("reminders")) {
        return { stats: { pending: 3 } };
      }
      return {};
    });
  });

  it("omits hidden items but keeps the customize link and locked settings entry", async () => {
    renderWithApp(<Sidebar />, "/invoices");
    expect(await screen.findByText("自定义导航")).toBeInTheDocument();
    expect(await screen.findByText("支付管理")).toBeInTheDocument();
    expect(screen.queryByText("发票管理")).not.toBeInTheDocument();
    expect(
      applyNavPreferences(
        toggleNavItemHidden(emptyNavPreferences(), "invoices", true),
        { isAdmin: true },
      )
        .flatMap((category) => category.items.map((item) => item.path)),
    ).not.toContain("/invoices");
  });
});
