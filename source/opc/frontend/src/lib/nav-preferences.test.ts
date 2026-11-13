import { describe, expect, it } from "vitest";
import {
  MENU_CATEGORIES,
  catalogItemCount,
  catalogItemById,
} from "@/lib/nav-catalog";
import {
  applyNavPreferences,
  emptyNavPreferences,
  moveNavCategory,
  moveNavItem,
  sanitizeNavPreferences,
  toggleNavItemHidden,
} from "@/lib/nav-preferences";

describe("nav catalog", () => {
  it("keeps the default five groups and 37 items", () => {
    expect(MENU_CATEGORIES).toHaveLength(5);
    expect(MENU_CATEGORIES.map((c) => c.label)).toEqual([
      "概览",
      "财务管理",
      "业务管理",
      "团队与档案",
      "系统",
    ]);
    expect(catalogItemCount()).toBe(37);
  });

  it("locks dashboard and settings so they cannot be hidden", () => {
    expect(catalogItemById("dashboard")?.locked).toBe(true);
    expect(catalogItemById("settings")?.locked).toBe(true);
  });
});

describe("sanitizeNavPreferences", () => {
  it("drops unknown, locked, and malformed ids", () => {
    const sanitized = sanitizeNavPreferences({
      version: 9,
      hidden_ids: ["invoices", "dashboard", "nope", "INVOICES", "", 1],
      category_order: ["business", "missing", "overview"],
      item_order: {
        finance: ["payments", "ghost", "invoices"],
      },
    });
    expect(sanitized.version).toBe(1);
    expect(sanitized.hidden_ids).toEqual(["invoices"]);
    expect(sanitized.category_order[0]).toBe("business");
    expect(sanitized.category_order).toContain("overview");
    expect(sanitized.category_order).toHaveLength(5);
    expect(sanitized.item_order.finance[0]).toBe("payments");
    expect(sanitized.item_order.finance[1]).toBe("invoices");
    expect(sanitized.item_order.finance).not.toContain("ghost");
  });
});

describe("applyNavPreferences", () => {
  it("defaults to the full catalog for admins", () => {
    const visible = applyNavPreferences(emptyNavPreferences(), { isAdmin: true });
    expect(visible).toHaveLength(5);
    expect(visible.flatMap((c) => c.items)).toHaveLength(37);
  });

  it("hides items and drops empty categories without removing routes from the catalog", () => {
    const hiddenTeam = sanitizeNavPreferences({
      hidden_ids: ["employees", "records", "reminders"],
    });
    const visible = applyNavPreferences(hiddenTeam, { isAdmin: true });
    expect(visible.map((c) => c.id)).not.toContain("team");
    expect(catalogItemById("employees")?.path).toBe("/employees");
  });

  it("reorders categories and items", () => {
    const prefs = sanitizeNavPreferences({
      category_order: ["system"],
      item_order: { overview: ["company-info", "dashboard"] },
    });
    const visible = applyNavPreferences(prefs, { isAdmin: true });
    expect(visible[0].id).toBe("system");
    const overview = visible.find((c) => c.id === "overview")!;
    expect(overview.items[0].id).toBe("company-info");
    expect(overview.items.map((i) => i.id)).toContain("dashboard");
  });

  it("filters admin-only items for non-admins", () => {
    const visible = applyNavPreferences(emptyNavPreferences(), { isAdmin: false });
    const ids = visible.flatMap((c) => c.items.map((i) => i.id));
    expect(ids).not.toContain("salary");
    expect(ids).not.toContain("user-management");
    expect(ids).not.toContain("operation-logs");
    expect(ids).toContain("settings");
  });
});

describe("preference mutators", () => {
  it("refuses to hide locked items and can restore a hidden item", () => {
    const blocked = toggleNavItemHidden(emptyNavPreferences(), "dashboard", true);
    expect(blocked.hidden_ids).toEqual([]);
    const hidden = toggleNavItemHidden(emptyNavPreferences(), "invoices", true);
    expect(hidden.hidden_ids).toEqual(["invoices"]);
    const shown = toggleNavItemHidden(hidden, "invoices", false);
    expect(shown.hidden_ids).toEqual([]);
  });

  it("moves categories and items within bounds", () => {
    const movedCat = moveNavCategory(emptyNavPreferences(), "overview", 1);
    expect(movedCat.category_order[0]).toBe("finance");
    expect(movedCat.category_order[1]).toBe("overview");
    const movedItem = moveNavItem(emptyNavPreferences(), "overview", "dashboard", 1);
    expect(movedItem.item_order.overview[0]).toBe("kpi-dashboard");
    expect(movedItem.item_order.overview[1]).toBe("dashboard");
    const unchanged = moveNavItem(emptyNavPreferences(), "overview", "dashboard", -1);
    expect(unchanged.item_order.overview[0]).toBe("dashboard");
  });
});
