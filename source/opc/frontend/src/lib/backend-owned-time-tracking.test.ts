import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned time tracking UI", () => {
  it("loads work duration, billable totals, and unbilled counts from the backend overview endpoint", () => {
    const source = readFileSync(
      resolve(root, "pages/TimeTracking.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/v1/time_entries/overview");
    expect(source).toContain("timeOverview.active_timer");
    expect(source).toContain("timeOverview.create_action");
    expect(source).toContain("activeTimer.start_action");
    expect(source).toContain("activeTimer.stop_action");
    expect(source).toContain("invoiceAction.action");
    expect(source).toContain("api.runAction(action");
    expect(source).not.toContain("api.post(API_ENDPOINTS.timeEntries");
    expect(source).not.toContain("/api/v1/time_entries/timer/start");
    expect(source).not.toContain("/api/v1/time_entries/timer/stop");
    expect(source).not.toContain("API_ENDPOINTS.timeEntries}/invoice");
    expect(source).toContain("timeOverview.billable_options");
    expect(source).toContain("entry.delete_action");
    expect(source).not.toContain("API_ENDPOINTS.timeEntries)");
    expect(source).not.toContain("API_ENDPOINTS.projects");
    expect(source).not.toContain("API_ENDPOINTS.clients");
    expect(source).not.toContain("getProjectName");
    expect(source).not.toContain("getClientName");
    expect(source).not.toContain("formatMinutes");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("Date.now");
    expect(source).not.toContain("setInterval");
    expect(source).not.toContain("timerStart");
    expect(source).not.toContain("formatElapsed");
    expect(source).not.toContain("new Date()");
    expect(source).not.toContain("start_time: timerStart");
    expect(source).not.toContain("end_time: endTime");
    expect(source).not.toContain("duration_minutes: durationMinutes");
    expect(source).not.toContain("date: timerStart.toISOString()");
    expect(source).not.toContain("const todayEntries =");
    expect(source).not.toContain("todayEntries.reduce");
    expect(source).not.toContain("entries.filter((e: any) => e.is_billable)");
    expect(source).not.toContain(
      "entries.filter((e: any) => e.is_billable && e.status !== 'invoiced')",
    );
    expect(source).not.toContain("entries.reduce((s: number, e: any)");
    expect(source).not.toContain('<SelectItem value="1">');
    expect(source).not.toContain('<SelectItem value="0">');
    expect(source).not.toContain('confirm("确定删除?")');
    expect(source).not.toContain("deleteEntry.mutate(entry.id)");
    expect(source).not.toContain("api.delete(`${API_ENDPOINTS.timeEntries}/${id}`)");
  });
});
