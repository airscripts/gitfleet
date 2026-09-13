# Gitfleet Site

Static Astro homepage for discovering Gitfleet, downloading releases, and
reaching the GitHub-hosted documentation.

The page keeps a fixed skeleton: header, ASCII hero, live terminal preview, and
footer. Visual language is sonar cyan on light and dark surfaces, with Plus
Jakarta Sans, IBM Plex Mono, and the Gitfleet hexagon mark.

## Requirements

- Node.js 24 or newer
- pnpm 11.15.0 or newer

The package manager is pinned in `package.json` (`packageManager`) and should
stay aligned with CI.

## Development

```bash
pnpm install
pnpm dev
```

`pnpm preview` serves the production `dist/` after `pnpm build`.

## Quality Gates

```bash
pnpm format:check
pnpm lint
pnpm test
pnpm build
pnpm test:integration
pnpm test:e2e
```

`pnpm verify` runs that sequence. Integration tests and Playwright both expect a
fresh `dist/` from `pnpm build`.

| Gate         | Tool                                           |
| ------------ | ---------------------------------------------- |
| Format       | Prettier (`printWidth` 100)                    |
| Lint / types | ESLint and `astro check`                       |
| Unit         | Vitest over `src/`                             |
| Build        | Astro static output                            |
| Integration  | Vitest over `tests/` against `dist/index.html` |
| End-to-end   | Playwright Chromium against `pnpm preview`     |

Automated tests must not call live GitHub or GitLab APIs. The homepage star
count is filled in the browser after load; unit and Playwright suites mock
`https://api.github.com/repos/airscripts/gitfleet`.

First-time Playwright setup on a machine:

```bash
pnpm exec playwright install chromium
```

CI installs Chromium with OS dependencies before `pnpm test:e2e`.

## Structure

- `src/layouts/` — document shell, metadata, and font loading.
- `src/components/` — header, hero, terminal, footer, and brand mark.
- `src/data/` — typed product copy and public URLs shared by pages and tests.
- `src/scripts/` — theme preference (light, dark, system), terminal typing, and
  GitHub star count.
- `src/styles/` — visual tokens. Fleet and signal cyan values are locked by
  `src/styles/global.test.ts`.
- `tests/` — build-output integration assertions.
- `e2e/` — Playwright coverage for keyboard navigation, theme toggle, and
  mocked star counts.
- `public/` — site-served copies of the logo, favicon, and cover.

Canonical brand files live in [`gitfleet-assets/`](../gitfleet-assets/):

| File                          | Role                                 |
| ----------------------------- | ------------------------------------ |
| `gitfleet-assets/cover.png`   | 1280×640 README and Open Graph cover |
| `gitfleet-assets/logo.png`    | 1024×1024 transparent hex mark       |
| `gitfleet-assets/favicon.png` | 64×64 favicon                        |

`public/logo.png` is a smaller header copy so the homepage does not ship the
full 1024px asset.

## Accessibility

The homepage exposes a skip link, header / main / footer landmarks, a visible
`h1`, and keyboard-visible focus rings. The footer control cycles light, dark,
and system color themes; `data-theme` stays `light` or `dark` for tokens, while
`data-theme-preference` records the selected mode. The terminal typewriter is
decorative (`aria-hidden`) so assistive technology is not flooded with character
updates.
The star count starts empty in HTML and is announced through the GitHub link
label after a successful client fetch.

Target Lighthouse scores for the production build are 95 or higher in
Performance, Accessibility, Best Practices, and SEO.

## Stack

- Astro 7, static output
- Tailwind CSS 4
- TypeScript
- Vitest 4
- Playwright

## License

MIT. See the repository [`LICENSE`](../LICENSE).
