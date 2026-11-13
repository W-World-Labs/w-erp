import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned reconciliation center UI", () => {
  it("renders backend-owned reconciliation sections instead of querying and formatting raw entities", () => {
    const source = readFileSync(
      resolve(root, "pages/ReconciliationCenter.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/v1/reconciliation/overview");
    expect(source).toContain("stat.class_name");
    expect(source).not.toContain("API_ENDPOINTS.bankTransactions");
    expect(source).not.toContain("API_ENDPOINTS.payments");
    expect(source).not.toContain("API_ENDPOINTS.invoices");
    expect(source).not.toContain("API_ENDPOINTS.salary");
    expect(source).not.toContain("API_ENDPOINTS.rent");
    expect(source).not.toContain("API_ENDPOINTS.cloudBills");
    expect(source).not.toContain("API_ENDPOINTS.softwareSubscriptions");
    expect(source).not.toContain("API_ENDPOINTS.bankTransactionsLegacy");
    expect(source).not.toContain("safeFormat");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("unmatchedCount =");
    expect(source).not.toContain("transaction.transaction_type ===");
    expect(source).not.toContain("payment.payment_type ===");
    expect(source).not.toContain("subscription.billing_cycle ===");
    expect(source).not.toContain("toneClasses");
  });
});
