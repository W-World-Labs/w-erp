import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned reminders UI", () => {
  it("loads grouping, date state, status labels, priority labels, and type labels from backend overview", () => {
    const source = readFileSync(resolve(root, "pages/Reminders.tsx"), "utf8");
    const notificationCenter = readFileSync(
      resolve(root, "components/NotificationCenter.tsx"),
      "utf8",
    );
    const sidebar = readFileSync(resolve(root, "components/Sidebar.tsx"), "utf8");
    const contracts = readFileSync(
      resolve(root, "lib/api/contracts.ts"),
      "utf8",
    );

    expect(source).toContain("API_ENDPOINTS.remindersOverview");
    expect(source).toContain("overview.check_asset_expiry_action");
    expect(source).toContain("overview.check_operating_deadlines_action");
    expect(source).toContain("reminder.complete_action");
    expect(source).toContain("reminder.dismiss_action");
    expect(source).toContain("api.runAction(action)");
    expect(source).toContain("api.runAction<{ reminders_created?: number }>");
    expect(notificationCenter).toContain("API_ENDPOINTS.remindersOverview");
    expect(sidebar).toContain("API_ENDPOINTS.remindersOverview");
    expect(sidebar).toContain("overview.stats.pending");
    expect(notificationCenter).toContain("overview.pending_reminders");
    expect(notificationCenter).toContain("reminder.remind_date_label");
    expect(notificationCenter).toContain("reminder.type_label");
    expect(notificationCenter).toContain("reminder.priority_badge");
    expect(contracts).toContain("remindersOverview");
    expect(contracts).toContain("remindersCheckOperatingDeadlines");

    expect(source).not.toContain('queryKey: ["reminders"]');
    expect(source).not.toContain("/api/reminders");
    expect(source).not.toContain("/complete");
    expect(source).not.toContain("/dismiss");
    expect(source).not.toContain("/api/ai/chat");
    expect(source).not.toContain("API_ENDPOINTS.remindersCheckAssetExpiry");
    expect(source).not.toContain("api.post");
    expect(source).not.toContain("invoke check-asset-expiry");
    expect(source).not.toContain('status: "completed"');
    expect(source).not.toContain('status: "dismissed"');
    expect(notificationCenter).not.toContain("/api/reminders");
    expect(sidebar).not.toContain("/api/reminders");
    expect(notificationCenter).not.toContain("getPriorityColor");
    expect(notificationCenter).not.toContain("getTypeLabel");
    expect(source).not.toContain(".filter(");
    expect(source).not.toContain("getStatusBadge");
    expect(source).not.toContain("getPriorityBadge");
    expect(source).not.toContain("getTypeLabel");
    expect(source).not.toContain("new Date");
    expect(source).not.toContain("isBefore");
    expect(source).not.toContain("isAfter");
    expect(source).not.toContain("addDays");
    expect(source).not.toContain("safeFormat");
    expect(notificationCenter).not.toContain("new Date");
    expect(notificationCenter).not.toContain("safeFormat");
  });
});
