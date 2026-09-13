# AGENTS.md

## Scope

- Path: gitfleet-site
- Parent: .
- Inheritance: additive

## Mission And Repository Map

This package owns the static Astro homepage for discovering Gitfleet.

- `src/layouts/` owns document metadata and the page shell.
- `src/components/` owns header, hero, terminal, footer, and brand mark.
- `src/data/` owns typed copy and public URLs.
- `src/scripts/` owns theme toggle, terminal typing, and the GitHub star count.
- `src/styles/global.css` owns visual tokens; fleet and signal cyan are locked by tests.
- `public/` serves resized copies of brand files from `../gitfleet-assets/`.

This package is not the CLI `gitfleet site` command family.

## Non-Negotiables

- Keep the header, ASCII hero, terminal, and footer skeleton.
- Fill the GitHub star count in the browser after load; keep `[data-star-count]` empty in static HTML.
- Keep the theme toggle in the footer after Sponsor.
- Update `README.md` when public site behavior or tests change.

## Don’ts

- Do not make live GitHub or GitLab requests from Vitest or Playwright.
- Do not change the `Star On GitHub` label or the locked sonar cyan token values.
- Do not move the theme toggle out of the footer link group or pin it with `fixed bottom-`.
- Do not replace the PNG mark with a reconstructed SVG.

## Quick Start

```bash
pnpm install
pnpm verify
```

Playwright needs `pnpm exec playwright install chromium` once locally.

## Change Routing

Homepage markup, styles, and browser behavior stay in this package. Canonical cover, logo, and favicon files stay in `../gitfleet-assets/`. CLI command docs stay in gitfleet-docs.

## Architecture Rules

Astro output is static. Node 24 and pnpm 11.15 are required. The star count uses the GitHub repository API from the browser only. The footer control cycles light, dark, and system; `data-theme` stays light or dark for tokens.

## Implementation Conventions

Format with Prettier at print width 100. Keep typed content in `src/data/site.ts`. Keep testable helpers such as star-count parsing in `src/scripts/`.

## Testing And Validation

Unit tests live beside `src/`. Integration tests in `tests/` read the Astro build output. Playwright in `e2e/` serves the production preview and must mock the GitHub stars API. Automated tests must not make live requests.

## Common Change Playbooks

For visual or layout work, update CSS tokens and viewport rules together with integration assertions. For star-count or theme behavior, update the helper, unit tests, and Playwright mocks. Keep skip-link, header, main, and footer landmarks intact for keyboard access.

## Free Region

No additional maintainer policy for this scope.

## Further Context

See [AGENTS.reference.md](AGENTS.reference.md) for provenance and package-local evidence. Parent guidance lives in the repository-root AGENTS.md.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
