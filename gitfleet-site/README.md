# Gitfleet Site

Static Astro homepage for Gitfleet.

## Requirements

- Node.js 24 or newer
- pnpm 11.15.0

## Development

```bash
pnpm install
pnpm dev
```

## Quality Gates

```bash
pnpm format:check
pnpm lint
pnpm test
pnpm build
pnpm test:integration
```

Use `pnpm verify` to run the full site gate locally. Integration tests expect a
fresh `dist/` from `pnpm build`.

`pnpm format:check` uses Prettier for formatting. `pnpm lint` uses ESLint for
source linting and `astro check` for Astro and TypeScript diagnostics.

## Structure

- `src/layouts/` contains page shells and document metadata.
- `src/components/` contains reusable interface sections.
- `src/data/` contains typed product content shared by pages and tests.
- `src/scripts/` contains browser behavior loaded by Astro.
- `tests/` contains build-output integration tests.

## Build

```bash
pnpm build
pnpm preview
```
