import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(__dirname, "..");

describe("backend-owned onboarding progress UI", () => {
  it("loads tracker and tutorial progress from one backend endpoint without local completion calculations", () => {
    const tracker = readFileSync(
      resolve(root, "components/tutorial/ProgressTracker.tsx"),
      "utf8",
    );
    const tutorial = readFileSync(
      resolve(root, "components/tutorial/OnboardingTutorial.tsx"),
      "utf8",
    );
    const contracts = readFileSync(resolve(root, "lib/api/contracts.ts"), "utf8");
    const combined = `${tracker}\n${tutorial}`;

    expect(contracts).toContain("onboardingProgress");
    expect(tracker).toContain("API_ENDPOINTS.onboardingProgress");
    expect(tutorial).toContain("API_ENDPOINTS.onboardingProgress");
    expect(combined).toContain("progress.modules");
    expect(combined).toContain("progress.tutorial_steps");
    expect(combined).toContain("progress.summary.progress_rate");
    expect(combined).toContain("progress.summary.progress_rate_label");
    expect(combined).toContain("progress.summary.completed_label");
    expect(combined).toContain("progress.summary.total_records_label");
    expect(combined).toContain("module.count_label");
    expect(combined).toContain("module.status_badge");
    expect(combined).toContain("step.completed");
    expect(combined).toContain("step.action_label");
    expect(combined).toContain("progress.incomplete_tip");
    expect(combined).toContain("progress.completed_tip");
    expect(tutorial).toContain("progress.summary.demo_company");
    expect(tutorial).toContain("progress.summary.tutorial_storage_key");
    expect(tutorial).toContain("progress.summary.activation_loop");
    expect(tutorial).toContain("activationLoop.steps");
    expect(tutorial).toContain("activationLoop.next_action");
    expect(tutorial).toContain("activationLoop.status_badge");
    expect(tutorial).toContain("current_step_id");
    expect(tutorial).toContain("evidence_label");
    expect(tutorial).toContain("progress.summary.operating_backbone");
    expect(tutorial).toContain("operatingBackbone.steps");
    expect(tutorial).toContain("operatingBackbone.next_action");
    expect(tutorial).toContain("operatingBackbone.status_badge");
    expect(tutorial).toContain("查看 Demo 教程");
    expect(tutorial).toContain("onboarding-tutorial-dismissed:global");

    expect(combined).not.toContain("tracker-company");
    expect(combined).not.toContain("tutorial-company");
    expect(tutorial).not.toContain("\"tutorial-dismissed\"");
    expect(combined).not.toContain("tracker-branches");
    expect(combined).not.toContain("tutorial-branches");
    expect(combined).not.toContain("const modules:");
    expect(combined).not.toContain("const tutorialSteps:");
    expect(combined).not.toContain("checkComplete");
    expect(combined).not.toContain("completedModules");
    expect(combined).not.toContain("completedSteps");
    expect(combined).not.toContain("modules.filter");
    expect(combined).not.toContain("modules.reduce");
    expect(combined).not.toContain("Math.round(progress)");
    expect(combined).not.toContain("new Set");
    expect(combined).not.toContain("api.get<{ data: any[]; count: number }>");
  });
});
