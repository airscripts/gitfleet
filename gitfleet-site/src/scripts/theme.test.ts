import { describe, expect, it } from "vitest";

import {
  applyThemePreference,
  nextThemePreference,
  readThemePreference,
  resolveRenderedTheme,
  themeButtonLabel,
} from "./theme";

describe("readThemePreference", () => {
  it("accepts stored light, dark, and system values", () => {
    expect(readThemePreference("light")).toBe("light");
    expect(readThemePreference("dark")).toBe("dark");
    expect(readThemePreference("system")).toBe("system");
  });

  it("defaults unknown or missing values to system", () => {
    expect(readThemePreference(null)).toBe("system");
    expect(readThemePreference("auto")).toBe("system");
  });
});

describe("nextThemePreference", () => {
  it("cycles light, dark, and system", () => {
    expect(nextThemePreference("light")).toBe("dark");
    expect(nextThemePreference("dark")).toBe("system");
    expect(nextThemePreference("system")).toBe("light");
  });
});

describe("resolveRenderedTheme", () => {
  it("follows the operating system only for the system preference", () => {
    expect(resolveRenderedTheme("system", true)).toBe("dark");
    expect(resolveRenderedTheme("system", false)).toBe("light");
    expect(resolveRenderedTheme("light", true)).toBe("light");
    expect(resolveRenderedTheme("dark", false)).toBe("dark");
  });
});

describe("themeButtonLabel", () => {
  it("names the current preference and the next preference", () => {
    expect(themeButtonLabel("system")).toBe("Color theme: system. Switch to light theme.");
    expect(themeButtonLabel("light")).toBe("Color theme: light. Switch to dark theme.");
    expect(themeButtonLabel("dark")).toBe("Color theme: dark. Switch to system theme.");
  });
});

describe("applyThemePreference", () => {
  it("stores the preference and the rendered light or dark theme", () => {
    const root = { dataset: {} } as HTMLElement;

    applyThemePreference(root, "system", true);

    expect(root.dataset.themePreference).toBe("system");
    expect(root.dataset.theme).toBe("dark");
  });
});
