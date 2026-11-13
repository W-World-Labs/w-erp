import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned deals UI", () => {
  it("loads pipeline stats, stage groups, client labels, and summaries from backend overview", () => {
    const source = readFileSync(
      resolve(root, "pages/DealsManagement.tsx"),
      "utf8",
    );
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.dealsOverview");
    expect(contracts).toContain("dealsOverview");
    expect(source).toContain("total_pipeline_label");
    expect(source).toContain("won_amount_label");
    expect(source).toContain("avg_probability_label");
    expect(source).toContain("stage.amount_label");
    expect(source).toContain("deal.amount_label");
    expect(source).toContain("deal.probability_label");
    expect(source).toContain("overview.create_action");
    expect(source).toContain("nextStage.action");
    expect(source).toContain("api.runAction(action");
    expect(source).toContain("action.default_values");

    expect(source).not.toContain("api.post(API_ENDPOINTS.deals");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.deals}/${id}`");
    expect(source).not.toContain('queryKey: ["deals"]');
    expect(source).not.toContain('queryKey: ["clients"]');
    expect(source).not.toContain("API_ENDPOINTS.clients");
    expect(source).not.toContain("const STAGES");
    expect(source).not.toContain("getClientName");
    expect(source).not.toContain(".filter(");
    expect(source).not.toContain(".reduce(");
    expect(source).not.toContain("Math.round");
    expect(source).not.toContain(".toFixed");
    expect(source).not.toContain("Number(deal.amount");
    expect(source).not.toContain("toLocaleString()");
  });

  it("keeps deal form amount and probability normalization on the backend", () => {
    const source = readFileSync(
      resolve(root, "pages/DealsManagement.tsx"),
      "utf8",
    );

    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("parseInt");
    expect(source).not.toContain("amount) values.amount");
    expect(source).not.toContain("probability) values.probability");
  });
});
