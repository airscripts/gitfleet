export function readStargazersCount(payload: unknown): number | null {
  if (payload === null || typeof payload !== "object") {
    return null;
  }

  const count = (payload as { stargazers_count?: unknown }).stargazers_count;

  if (typeof count !== "number" || !Number.isFinite(count) || count < 0) {
    return null;
  }

  return Math.floor(count);
}

export function formatStarCount(count: number): string {
  if (!Number.isFinite(count) || count < 0) {
    return "";
  }

  const wholeCount = Math.floor(count);

  if (wholeCount < 1000) {
    return String(wholeCount);
  }

  return new Intl.NumberFormat("en", {
    notation: "compact",
    maximumFractionDigits: 1,
  }).format(wholeCount);
}

export async function loadStargazersCount(
  fetchImpl: typeof fetch,
  url: string,
): Promise<number | null> {
  try {
    const response = await fetchImpl(url, {
      headers: { Accept: "application/vnd.github+json" },
    });

    if (!response.ok) {
      return null;
    }

    return readStargazersCount(await response.json());
  } catch {
    return null;
  }
}
