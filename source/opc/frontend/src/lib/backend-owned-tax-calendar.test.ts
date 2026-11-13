import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned tax calendar UI", () => {
  it("loads grouped calendar and filtered deadlines from backend overview", () => {
    const source = readFileSync(resolve(root, "pages/TaxCalendar.tsx"), "utf8");
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.taxCalendarOverview");
    expect(contracts).toContain("taxCalendarOverview");
    expect(source).toContain("overview.seed_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("overview.country_options");
    expect(source).toContain("overview.month_options");
    expect(source).toContain("event.country_flag");
    expect(source).toContain("event.tax_type_color");
    expect(source).toContain("deadline.tax_type_label");
    expect(source).toContain("deadline.recurrence_label");
    expect(source).toContain("deadline.applies_to_label");
    expect(source).toContain("monthBucket.label");
    expect(source).toContain("monthBucket.card_class_name");
    expect(source).toContain("monthBucket.count_badge");
    expect(source).toContain("monthBucket.visible_events");
    expect(source).toContain("monthBucket.overflow_label");
    expect(source).toContain("monthBucket.empty_message");
    expect(source).toContain("jurisdiction.country_flag");
    expect(source).toContain("jurisdiction.jurisdiction_type_badge");
    expect(source).toContain("jurisdiction.state_income_tax_badge");
    expect(source).toContain("jurisdiction.corporate_tax_rate_label");
    expect(source).toContain("jurisdiction.personal_tax_rate_label");
    expect(source).toContain("jurisdiction.vat_rate_label");
    expect(source).toContain("jurisdiction.sales_tax_rate_label");
    expect(source).not.toContain("useMemo");
    expect(source).not.toContain("allDeadlines.filter");
    expect(source).not.toContain("jurisdictions.filter");
    expect(source).not.toContain("new Date().getMonth");
    expect(source).not.toContain("calendarByMonth");
    expect(source).not.toContain("upcomingDeadlines");
    expect(source).not.toContain("MONTH_NAMES");
    expect(source).not.toContain("monthBucket.month === currentMonth");
    expect(source).not.toContain("events.slice(0, 6)");
    expect(source).not.toContain("events.length > 6");
    expect(source).not.toContain("events.length - 6");
    expect(source).not.toContain("COUNTRY_FLAGS");
    expect(source).not.toContain("TAX_TYPE_LABELS");
    expect(source).not.toContain("TAX_TYPE_COLORS");
    expect(source).not.toContain("jurisdiction.jurisdiction_type ===");
    expect(source).not.toContain("jurisdiction.corporate_tax_rate != null");
    expect(source).not.toContain("jurisdiction.personal_tax_rate != null");
    expect(source).not.toContain("jurisdiction.vat_rate != null");
    expect(source).not.toContain("jurisdiction.sales_tax_rate != null");
    expect(source).not.toContain("yearly: \"每年\"");
    expect(source).not.toContain("corporation: \"企业\"");
    expect(source).not.toContain('<SelectItem value="CN">');
    expect(source).not.toContain('<SelectItem value="US-states">');
    expect(source).not.toContain("api.post(API_ENDPOINTS.taxCalendarSeed");
  });
});
