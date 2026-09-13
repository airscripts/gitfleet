import { describe, expect, it, vi } from "vitest";

import { formatStarCount, loadStargazersCount, readStargazersCount } from "./stars";

describe("readStargazersCount", () => {
  it("reads a finite non-negative GitHub stargazers_count", () => {
    expect(readStargazersCount({ stargazers_count: 1284 })).toBe(1284);
    expect(readStargazersCount({ stargazers_count: 0 })).toBe(0);
    expect(readStargazersCount({ stargazers_count: 12.9 })).toBe(12);
  });

  it("rejects payloads that are not a usable GitHub repository document", () => {
    expect(readStargazersCount(null)).toBeNull();
    expect(readStargazersCount("1284")).toBeNull();
    expect(readStargazersCount({})).toBeNull();
    expect(readStargazersCount({ stargazers_count: -1 })).toBeNull();
    expect(readStargazersCount({ stargazers_count: Number.NaN })).toBeNull();
    expect(readStargazersCount({ stargazers_count: "12" })).toBeNull();
  });
});

describe("formatStarCount", () => {
  it("keeps small counts exact and compact-formats thousands", () => {
    expect(formatStarCount(0)).toBe("0");
    expect(formatStarCount(999)).toBe("999");
    expect(formatStarCount(1000)).toBe("1K");
    expect(formatStarCount(1284)).toBe("1.3K");
  });

  it("returns an empty label for unusable counts", () => {
    expect(formatStarCount(-1)).toBe("");
    expect(formatStarCount(Number.NaN)).toBe("");
  });
});

describe("loadStargazersCount", () => {
  it("returns the count from a successful GitHub repository payload", async () => {
    const fetchImpl = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ stargazers_count: 42 }),
    });

    await expect(
      loadStargazersCount(fetchImpl, "https://api.github.com/repos/airscripts/gitfleet"),
    ).resolves.toBe(42);

    expect(fetchImpl).toHaveBeenCalledWith("https://api.github.com/repos/airscripts/gitfleet", {
      headers: { Accept: "application/vnd.github+json" },
    });
  });

  it("returns null when the request fails or the payload is unusable", async () => {
    await expect(
      loadStargazersCount(
        vi.fn().mockResolvedValue({
          ok: false,
          json: async () => ({ stargazers_count: 12 }),
        }),
        "https://api.github.com/repos/airscripts/gitfleet",
      ),
    ).resolves.toBeNull();

    await expect(
      loadStargazersCount(
        vi.fn().mockRejectedValue(new Error("offline")),
        "https://api.github.com/repos/airscripts/gitfleet",
      ),
    ).resolves.toBeNull();
  });
});
