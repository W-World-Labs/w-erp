import { describe, expect, it } from "vitest";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(__dirname, "..");

describe("backend-owned collection automation", () => {
  it("keeps mutations in the collection hook and mounts the settings card", () => {
    const hook = fs.readFileSync(
      path.join(root, "hooks/useCollectionAutomation.ts"),
      "utf8",
    );
    const card = fs.readFileSync(
      path.join(root, "components/settings/CollectionAutomationCard.tsx"),
      "utf8",
    );
    const settings = fs.readFileSync(path.join(root, "pages/Settings.tsx"), "utf8");

    expect(hook).toContain('const endpoint = "/api/v1/settings/collections"');
    expect(hook).toContain("api.put<CollectionSettings>");
    expect(card).toContain("useCollectionAutomation");
    expect(card).not.toMatch(/api\.(post|put|delete)\s*\(/);
    expect(settings).toContain("<CollectionAutomationCard />");
  });

  it("renders delivery history in AR and credit derived from collections in CRM", () => {
    const aging = fs.readFileSync(
      path.join(root, "components/payments/AccountsReceivableAgingPanel.tsx"),
      "utf8",
    );
    const clients = fs.readFileSync(
      path.join(root, "pages/ClientManagement.tsx"),
      "utf8",
    );

    expect(aging).toContain("collection_history_count");
    expect(aging).toContain("自动催收历史");
    expect(clients).toContain("credit_profile");
    expect(clients).toContain("回款信用评分");
  });
});
