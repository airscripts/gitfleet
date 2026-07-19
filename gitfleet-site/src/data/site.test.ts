import { describe, expect, it } from "vitest";

import { bannerLines, docsBlobUrl, docsTreeUrl, releaseUrl, repoUrl, terminalTips } from "./site";

describe("site content", () => {
  it("keeps public links anchored to the Gitfleet repository", () => {
    expect(repoUrl).toBe("https://github.com/airscripts/gitfleet");
    expect(docsTreeUrl).toBe(`${repoUrl}/tree/main/gitfleet-docs`);
    expect(docsBlobUrl).toBe(`${repoUrl}/blob/main/gitfleet-docs`);
    expect(releaseUrl).toBe(`${repoUrl}/releases`);
  });

  it("keeps the CLI banner stable enough for the homepage hero", () => {
    expect(bannerLines).toHaveLength(6);
    expect(bannerLines[0]).toContain("██████");
    expect(bannerLines.join("\n")).toContain("████████");
  });

  it("keeps terminal tips useful without pre-rendering visible output", () => {
    expect(terminalTips).toHaveLength(25);
    expect(terminalTips).toContain("Clone a whole owner fleet with gf repo clone --all.");
    expect(new Set(terminalTips).size).toBe(terminalTips.length);
    expect(terminalTips.every((tip) => tip.length > 24)).toBe(true);
  });
});
