import { describe, expect, it } from "vitest";
import { readdirSync, readFileSync } from "node:fs";
import { relative, resolve } from "node:path";

const root = resolve(__dirname, "..");

function sourceFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(dir, entry.name);
    if (entry.isDirectory()) {
      return sourceFiles(path);
    }
    if (entry.isFile() && /\.(ts|tsx)$/.test(entry.name)) {
      return [path];
    }
    return [];
  });
}

describe("backend-owned action boundary", () => {
  it("keeps direct network mutations and raw fetches out of pages and components", () => {
    const directNetworkPattern =
      /api\.(post|put|delete)(?:<[^>]+>)?\(|\bfetch\(|XMLHttpRequest|axios\./;
    const offenders = [resolve(root, "pages"), resolve(root, "components")]
      .flatMap(sourceFiles)
      .filter((path) => directNetworkPattern.test(readFileSync(path, "utf8")))
      .map((path) => relative(root, path));

    expect(offenders).toEqual([]);
  });
});
