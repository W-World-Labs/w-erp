import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const frontendRoot = resolve(__dirname, "../..");

describe("Vite SPA asset output", () => {
  it("does not emit hashed bundles under /assets because that path is an API route", () => {
    const config = readFileSync(resolve(frontendRoot, "vite.config.ts"), "utf8");

    expect(config).toContain("assetsDir");
    expect(config).toContain("app-assets");
    expect(config).not.toContain('assetsDir: "assets"');
  });
});
