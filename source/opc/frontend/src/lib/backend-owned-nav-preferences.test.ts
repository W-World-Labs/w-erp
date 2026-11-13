import { describe, expect, it } from "vitest";
import fs from "node:fs";
import path from "node:path";

const root = path.resolve(__dirname, "..");

describe("backend-owned nav preferences", () => {
  it("loads and saves sidebar prefs through the settings endpoint", () => {
    const hook = fs.readFileSync(path.join(root, "hooks/useNavPreferences.ts"), "utf8");
    const card = fs.readFileSync(
      path.join(root, "components/settings/NavigationPreferencesCard.tsx"),
      "utf8",
    );
    const settings = fs.readFileSync(path.join(root, "pages/Settings.tsx"), "utf8");
    const sidebar = fs.readFileSync(path.join(root, "components/Sidebar.tsx"), "utf8");
    const contracts = fs.readFileSync(path.join(root, "lib/api/contracts.ts"), "utf8");

    expect(contracts).toContain('navPreferences: "/api/v1/settings/nav"');
    expect(hook).toContain("API_ENDPOINTS.navPreferences");
    expect(hook).toContain("api.put<unknown>(API_ENDPOINTS.navPreferences");
    expect(card).toContain("useNavPreferences");
    expect(card).not.toMatch(/api\.(get|post|put|delete)\s*\(/);
    expect(settings).toContain("<NavigationPreferencesCard />");
    expect(sidebar).toContain("useNavPreferences");
    expect(sidebar).toContain("categories");
    expect(sidebar).not.toContain("const menuCategories");
  });
});
