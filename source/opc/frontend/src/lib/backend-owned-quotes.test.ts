import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned quotes UI", () => {
  it("loads quote stats, client labels, status labels, and next number from backend overview", () => {
    const source = readFileSync(
      resolve(root, "pages/QuotesManagement.tsx"),
      "utf8",
    );
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.quotesOverview");
    expect(contracts).toContain("quotesOverview");
    expect(source).toContain("total_amount_label");
    expect(source).toContain("quote.total_label");
    expect(source).toContain("quote.status_badge");
    expect(source).toContain("quote.valid_until_label");
    expect(source).toContain("quote.delete_action");
    expect(source).toContain("overview.create_action");
    expect(source).toContain("editingQuote.update_action");
    expect(source).toContain("api.runAction(action");

    expect(source).not.toContain("api.post(API_ENDPOINTS.quotes");
    expect(source).not.toContain("api.put(`${API_ENDPOINTS.quotes}/${id}`");
    expect(source).not.toContain('queryKey: ["quotes"]');
    expect(source).not.toContain('queryKey: ["clients"]');
    expect(source).not.toContain("API_ENDPOINTS.clients");
    expect(source).not.toContain("new Date().getFullYear");
    expect(source).not.toContain("quotes.filter");
    expect(source).not.toContain(".reduce(");
    expect(source).not.toContain("getClientName");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain('quote.status === "accepted"');
    expect(source).not.toContain(".toFixed");
    expect(source).not.toContain("Number(quote.total");
    expect(source).not.toContain("toLocaleString()");
    expect(source).not.toContain('confirm("确定删除?")');
    expect(source).not.toContain("deleteQuote.mutate(quote.id)");
    expect(source).not.toContain("api.delete(`${API_ENDPOINTS.quotes}/${id}`)");
  });

  it("keeps quote form totals and tax normalization on the backend", () => {
    const source = readFileSync(
      resolve(root, "pages/QuotesManagement.tsx"),
      "utf8",
    );

    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("values[field] =");
  });
});
