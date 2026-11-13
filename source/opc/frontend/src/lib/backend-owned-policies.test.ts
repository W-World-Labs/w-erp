import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned policy UI", () => {
  it("renders policy dashboard display data from backend without local filtering or formatting", () => {
    const source = readFileSync(resolve(root, "pages/PolicyDashboard.tsx"), "utf8");

    expect(source).toContain("/api/v1/policies/dashboard/overview");
    expect(source).toContain("category_options");
    expect(source).toContain("country_options");
    expect(source).toContain("overview?.policies");
    expect(source).toContain("overview?.matched_policies");
    expect(source).toContain("overview?.applications");
    expect(source).toContain("p.category_badge");
    expect(source).toContain("p.country_badge");
    expect(source).toContain("p.amount_max_label");
    expect(source).toContain("match.policy");
    expect(source).toContain("match.match_score_label");
    expect(source).toContain("match.rank_badge");
    expect(source).toContain("match.card_class_name");
    expect(source).toContain("match.rank_badge?.visible");
    expect(source).toContain("p.apply_action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("app.status_badge");
    expect(source).toContain("app.updated_at_label");
    expect(source).not.toContain("policiesData?.filter");
    expect(source).not.toContain("categoryLabels");
    expect(source).not.toContain("statusMap");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("safeFormat");
    expect(source).not.toContain("date-fns");
    expect(source).not.toContain("/api/v1/policies/match");
    expect(source).not.toContain("/api/v1/policies/applications/overview");
    expect(source).not.toContain("api.get<{ data: any[]; count: number");
    expect(source).not.toContain('status: "preparing"');
    expect(source).not.toContain("Auto-added from Policy Dashboard");
    expect(source).not.toContain("api.post('/api/v1/policies/applications'");
    expect(source).not.toContain('match.rank_badge?.label === "最优匹配"');
  });

  it("renders policy detail display data from backend without parsing business fields locally", () => {
    const source = readFileSync(resolve(root, "pages/PolicyDetail.tsx"), "utf8");

    expect(source).toContain("policy.category_badge");
    expect(source).toContain("policy.country_badge");
    expect(source).toContain("policy.amount_range_label");
    expect(source).toContain("policy.equity_required_label");
    expect(source).toContain("policy.eligibility_items");
    expect(source).toContain("policy.benefit_items");
    expect(source).toContain("policy.match_profile_items");
    expect(source).toContain("policy.apply_action");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain('/api/v1/policies/applications');
    expect(source).not.toContain("countryFlags");
    expect(source).not.toContain("countryNames");
    expect(source).not.toContain("categoryLabels");
    expect(source).not.toContain("parseListItems");
    expect(source).not.toContain("formatAmount");
    expect(source).not.toContain("formatJsonList");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("toFixed");
    expect(source).not.toContain("Math.round");
    expect(source).not.toContain('status: "preparing"');
    expect(source).not.toContain("policy_name:");
    expect(source).not.toContain("notes:");
    expect(source).not.toContain("api.post(\"/api/v1/policies/applications\"");
  });
});
