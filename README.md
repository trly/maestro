# Maestro

AI-powered orchestrator UI for running prompts across multiple repositories.

## Setup

1. Install dependencies:
```bash
bun install
```

2. Configure environment variables:
```bash
cp .env.example .env
```

Edit `.env` and add your GitHub Personal Access Token:
- Generate at: https://github.com/settings/tokens
- Required scopes: `repo`

## Development

Start the dev server:
```bash
bun run dev
```

## Commands

- `bun run dev` - Start SvelteKit dev server
- `bun run build` - Production build
- `bun run check` - Run svelte-check
- `bun test` - Run tests

## Tech Stack

SvelteKit 2 + Svelte 5, Tailwind 4, TypeScript, Bun
