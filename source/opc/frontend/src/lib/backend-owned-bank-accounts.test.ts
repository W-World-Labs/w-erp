import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned bank accounts UI", () => {
  it("loads balances, transaction summaries, account type labels, and status labels from backend overview", () => {
    const source = readFileSync(
      resolve(root, "pages/BankAccounts.tsx"),
      "utf8",
    );
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.bankAccountsOverview");
    expect(contracts).toContain("bankAccountsOverview");

    expect(source).not.toContain('queryKey: ["bankAccounts"]');
    expect(source).not.toContain('queryKey: ["transactionStats"]');
    expect(source).not.toContain("transactionStats");
    expect(source).not.toContain(".reduce(");
    expect(source).not.toContain(".filter(");
    expect(source).not.toContain("account.status ===");
    expect(source).not.toContain("account.account_type ===");
    expect(source).not.toContain("toLocaleString");
  });
});
