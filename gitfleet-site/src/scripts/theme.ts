export const THEME_STORAGE_KEY = "gitfleet-site-theme";

export type ThemePreference = "light" | "dark" | "system";
export type RenderedTheme = "light" | "dark";

const THEME_PREFERENCES: readonly ThemePreference[] = ["light", "dark", "system"];

export function readThemePreference(stored: string | null): ThemePreference {
  return THEME_PREFERENCES.find((preference) => preference === stored) ?? "system";
}

export function nextThemePreference(preference: ThemePreference): ThemePreference {
  const index = THEME_PREFERENCES.indexOf(preference);

  return THEME_PREFERENCES[(index + 1) % THEME_PREFERENCES.length] ?? "system";
}

export function resolveRenderedTheme(
  preference: ThemePreference,
  prefersDark: boolean,
): RenderedTheme {
  if (preference === "system") {
    return prefersDark ? "dark" : "light";
  }

  return preference;
}

export function themeButtonLabel(preference: ThemePreference): string {
  return `Color theme: ${preference}. Switch to ${nextThemePreference(preference)} theme.`;
}

export function applyThemePreference(
  root: HTMLElement,
  preference: ThemePreference,
  prefersDark: boolean,
): void {
  root.dataset.themePreference = preference;
  root.dataset.theme = resolveRenderedTheme(preference, prefersDark);
}
