import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned system status UI", () => {
  it("renders system health labels, badges, counts, and percentages supplied by the backend", () => {
    const source = readFileSync(
      resolve(root, "components/settings/SystemStatusCard.tsx"),
      "utf8",
    );

    expect(source).toContain("/api/system/status");
    expect(source).toContain("data.health.status_badge");
    expect(source).toContain("data.health.database_badge");
    expect(source).toContain("data.health.timestamp_label");
    expect(source).toContain("data.database.file_size_label");
    expect(source).toContain("data.database.total_rows_label");
    expect(source).toContain("data.database.health_percent");
    expect(source).toContain("data.database.health_percent_label");
    expect(source).toContain("data.database.summary_label");
    expect(source).toContain("data.storage.data_files_label");
    expect(source).toContain("data.api.summary_label");
    expect(source).toContain("data.api.health_percent_label");
    expect(source).toContain("t.count_label");
    expect(source).toContain("t.status_badge");
    expect(source).toContain("ep.status_badge");

    expect(source).not.toContain("function formatBytes");
    expect(source).not.toContain("function formatTimestamp");
    expect(source).not.toContain("toLocaleString");
    expect(source).not.toContain("Math.round");
    expect(source).not.toContain("Math.pow");
    expect(source).not.toContain("parseFloat");
    expect(source).not.toContain("apiChecks");
    expect(source).not.toContain("checkAllApis");
    expect(source).not.toContain("Object.values");
    expect(source).not.toContain("api.get(url)");
    expect(source).not.toContain("fetch(`${apiBase}/health`)");
    expect(source).not.toContain(".filter(t => t.status");
  });
});
