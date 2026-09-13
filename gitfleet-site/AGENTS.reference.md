# AGENTS.reference.md

## Provenance And Decisions

- Agentskill Version: `2.1.0`.
- Evidence Schema Version: `4`.
- Repository Revision: `7f407aead045dabd74f15c664b42fac5113cfffd`.
- Configuration: inherited from the repository root; default signature enabled.
- Maintainer-Confirmed Decisions: adopt gitfleet-site as a nested scope; keep the homepage skeleton; fetch GitHub stars only in the browser; mock that API in Vitest and Playwright; keep sonar cyan tokens locked; cycle light, dark, and system themes from the footer control; use PNG brand assets from `../gitfleet-assets/` with a smaller header copy in `public/`.
- Unresolved Uncertainty: none specific to this package beyond the root merge-strategy gap.

## Boundaries And Ownership

This package is the nearest owner of the Astro homepage, Tailwind tokens, browser scripts, and Node quality gates. Canonical brand files live in `../gitfleet-assets/`. The CLI `site` family is documented under `../gitfleet-docs/commands/site.md`, not this homepage.

## Testing Topology

- `pnpm test` runs Vitest over `src/`.
- `pnpm test:integration` reads the Astro build output after `pnpm build`.
- `pnpm test:e2e` runs Playwright Chromium against `pnpm preview` on port 4173 and must mock the GitHub repository API.
- Integration HTML assertions currently require the document title, description, docs/releases/repo URLs, `Download Releases`, `Star On GitHub`, empty `data-terminal-tip` and `data-star-count`, terminal JSON, `airscript@gitfleet:~$`, skip-link landmarks, and the theme toggle after `Sponsor`.

## Evidence Summary

`package.json` pins Node 24, pnpm 11.15, Astro 7, Tailwind 4, Vitest, and Playwright. `src/styles/global.test.ts` locks `--site-fleet` / `--site-signal` to `#007c91` in light theme and `#22d3ee` in dark theme. The homepage is a single static route in `src/pages/index.astro`.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
