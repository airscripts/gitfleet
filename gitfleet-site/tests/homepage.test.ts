import { readFile } from "node:fs/promises";

import { describe, expect, it } from "vitest";

const html = await readFile(new URL("../dist/index.html", import.meta.url), "utf8");

describe("homepage build output", () => {
  it("uses the expected document metadata", () => {
    expect(html).toContain("<title>Gitfleet</title>");
    expect(html).toContain(
      'content="Gitfleet is a provider-neutral CLI for managing repositories across GitHub and GitLab."',
    );
  });

  it("renders the primary navigation and calls to action", () => {
    expect(html).toContain('href="https://github.com/airscripts/gitfleet/tree/main/gitfleet-docs"');
    expect(html).toContain('href="https://github.com/airscripts/gitfleet/releases"');
    expect(html).toContain('href="https://github.com/airscripts/gitfleet"');
    expect(html).toContain("Download Releases");
    expect(html).toContain("Star On GitHub");
  });

  it("keeps terminal output empty until the client script starts typing", () => {
    expect(html).toContain("<span data-terminal-tip></span>");
    expect(html).toContain('<script id="terminal-tips" type="application/json">');
    expect(html).toContain("airscript@gitfleet:~$");
  });

  it("keeps the theme toggle in the footer link group", () => {
    const sponsorIndex = html.indexOf("Sponsor</a>");
    const toggleIndex = html.indexOf("data-theme-toggle", sponsorIndex);

    expect(sponsorIndex).toBeGreaterThan(-1);
    expect(toggleIndex).toBeGreaterThan(sponsorIndex);
    expect(html).not.toContain("fixed bottom-");
  });
});
