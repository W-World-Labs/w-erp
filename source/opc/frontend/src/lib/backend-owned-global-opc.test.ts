import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned Global OPC UI", () => {
  it("renders hub and calculator display fields from backend", () => {
    const source = readFileSync(resolve(root, "pages/GlobalOpcHub.tsx"), "utf8");

    expect(source).toContain("hub.flag");
    expect(source).toContain("hub.corporate_tax_rate_label");
    expect(source).toContain("hub.dividend_tax_rate_label");
    expect(source).toContain("hub.rent_label");
    expect(source).toContain("hub.feature_badges");
    expect(source).toContain("r.net_take_home_label");
    expect(source).toContain("r.take_home_percentage_label");
    expect(source).toContain("r.mrr_survival_line_label");
    expect(source).toContain("r.friction_costs_label");
    expect(source).toContain("r.living_costs_label");
    expect(source).toContain("r.total_tax_paid_label");
    expect(source).toContain("r.setup_and_maintenance_roi_label");
    expect(source).toContain("r.tax_type_badge");
    expect(source).toContain("r.is_best");
    expect(source).not.toContain("FLAG_MAP");
    expect(source).not.toContain("TAX_TYPE_COLORS");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("toFixed");
    expect(source).not.toContain("getBestHub");
    expect(source).not.toContain(".sort(");
    expect(source).not.toContain("tax_sandbox_type.includes");
  });
});
