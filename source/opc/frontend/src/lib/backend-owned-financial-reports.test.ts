import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned financial health report", () => {
  it("renders backend report output without recalculating health score in the page", () => {
    const source = readFileSync(
      resolve(root, "pages/FinancialReports.tsx"),
      "utf8",
    );
    const hook = readFileSync(
      resolve(root, "hooks/useMonthlyClose.ts"),
      "utf8",
    );

    expect(source).toContain("/api/v1/reports/financial-health");
    expect(source).toContain("useMonthlyClose");
    expect(source).toContain("生成月度报告");
    expect(source).toContain("type=\"month\"");
    expect(source).toContain("下载");
    expect(source).not.toContain("api.post");
    expect(hook).toContain("API_ENDPOINTS.monthlyClose");
    expect(hook).toContain("api.post<MonthlyCloseResult>");
    expect(source).toContain("healthScore.badge");
    expect(source).toContain("asset_liability_ratio_label");
    expect(source).toContain("operating_cash_flow_label");
    expect(source).toContain("operating_cash_flow_trend");
    expect(source).toContain("rec.variant");
    expect(source).toContain("rec.icon");
    expect(source).not.toContain("API_ENDPOINTS.assets");
    expect(source).not.toContain("API_ENDPOINTS.invoices");
    expect(source).not.toContain("API_ENDPOINTS.payments");
    expect(source).not.toContain("API_ENDPOINTS.salary");
    expect(source).not.toContain("calculateMetrics");
    expect(source).not.toContain("calculateHealthScore");
    expect(source).not.toContain("getRecommendations");
    expect(source).not.toContain("assetLiabilityScore");
    expect(source).not.toContain("profitabilityScore");
    expect(source).not.toContain("getScoreBadge");
    expect(source).not.toContain(".toFixed");
    expect(source).not.toContain("profit_margin >= 0");
    expect(source).not.toContain("operating_cash_flow >= 0");
    expect(source).not.toContain("fullMark: point.full_mark");
    expect(source).not.toContain("rec.type ===");
  });
});
