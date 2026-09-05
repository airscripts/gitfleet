import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

const stylesheet = readFileSync(new URL("./global.css", import.meta.url), "utf8");

describe("site color tokens", () => {
  it("keeps fleet and signal mapped as semantic tokens", () => {
    expect(stylesheet).toContain("--color-fleet: var(--site-fleet);");
    expect(stylesheet).toContain("--color-signal: var(--site-signal);");
  });

  it("locks the sonar cyan values for both themes", () => {
    expect(stylesheet).toContain("--site-fleet: #007c91;");
    expect(stylesheet).toContain("--site-signal: #007c91;");
    expect(stylesheet).toContain("--site-fleet: #22d3ee;");
    expect(stylesheet).toContain("--site-signal: #22d3ee;");
  });
});
