export const repoUrl = "https://github.com/airscripts/gitfleet";
export const docsTreeUrl = `${repoUrl}/tree/main/gitfleet-docs`;
export const docsBlobUrl = `${repoUrl}/blob/main/gitfleet-docs`;
export const releaseUrl = `${repoUrl}/releases`;

export const bannerLines = [
  " ██████╗ ██╗████████╗███████╗██╗     ███████╗███████╗████████╗",
  "██╔════╝ ██║╚══██╔══╝██╔════╝██║     ██╔════╝██╔════╝╚══██╔══╝",
  "██║  ███╗██║   ██║   █████╗  ██║     █████╗  █████╗     ██║   ",
  "██║   ██║██║   ██║   ██╔══╝  ██║     ██╔══╝  ██╔══╝     ██║   ",
  "╚██████╔╝██║   ██║   ██║     ███████╗███████╗███████╗   ██║   ",
  " ╚═════╝ ╚═╝   ╚═╝   ╚═╝     ╚══════╝╚══════╝╚══════╝   ╚═╝   ",
];

export const banner = bannerLines.join("\n");

export const terminalTips = [
  "Install Gitfleet with Cargo and start managing repositories from one small CLI.",
  "Use one provider-neutral vocabulary across GitHub and GitLab.",
  "Switch to --json when the same workflow needs to run in automation.",
  "Run gf auth status when you need to see the active profile and token state.",
  "Clone a whole owner fleet with gf repo clone --all.",
  "Scope bulk cloning with exactly one of --org or --user.",
  "Preview bulk repository clones with --dry-run before writing directories.",
  "Use --include-forks only when forked repositories belong in your fleet clone.",
  "Use --include-archived only when archived repositories still matter locally.",
  "Limit repository clone history with --depth when you only need recent commits.",
  "Gitfleet skips existing clone destinations instead of overwriting local work.",
  "Paginated list and search commands accept --page for concrete provider pages.",
  "Human-readable output is the default for interactive terminal work.",
  "Structured output is explicit so scripts can depend on --json.",
  "Provider capability checks prevent pretending GitHub and GitLab are identical.",
  "Gitfleet reports unsupported provider behavior instead of emulating it silently.",
  "Configuration lives under ~/.config/gitfleet/ in TOML format.",
  "Environment variables use the GITFLEET_ prefix.",
  "The gitfleet and gf binaries expose the same command surface.",
  "Destructive human-mode operations require confirmation.",
  "Non-interactive destructive operations require an explicit --yes.",
  "Bulk mutations provide dry-run previews when a meaningful preview exists.",
  "Provider tokens can come from the active profile or environment.",
  "Gitfleet auth login validates the token before saving it.",
  "The docs live in gitfleet-docs with command, workflow, provider, and safety pages.",
];
