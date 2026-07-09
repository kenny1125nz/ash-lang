# CI Pipeline — Deploy Website & Build-and-Test Updates

## Background

The current CI in `.github/workflows/ci.yml` has a `deploy-website` job (triggered on tags) that builds WASM and deploys static HTML files via `wrangler pages deploy web-site`. The new Astro-based site requires:
- WASM output copied to `web-site/public/wasm/` (not `web-site/wasm/`)
- `docs/assemble.js` to generate the reference fragment
- D1 migrations applied before build
- `npm ci && npm test && npm run build` in `web-site/`
- Deploy from `web-site/dist/` (Astro's output) instead of `web-site/`

The `build-and-test` job (triggered on push to master) must also commit the new reference fragment.

## Intended Solution

### `deploy-website` Job

Replace the current `deploy-website` job with:

```yaml
deploy-website:
  name: Deploy Website
  if: github.ref_type == 'tag'
  needs: github-release
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: wasm32-unknown-unknown

    - name: Install wasm-pack
      run: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

    - name: Build WASM
      run: |
        cd ash-wasm
        wasm-pack build --target web --release
        mkdir -p ../web-site/public/wasm
        cp pkg/*.wasm ../web-site/public/wasm/
        cp pkg/*.js ../web-site/public/wasm/

    - name: Generate docs
      run: |
        npm install --prefix docs
        node docs/assemble.js

    - name: Setup Node
      uses: actions/setup-node@v4
      with:
        node-version: 20

    - name: Apply D1 migrations
      env:
        CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
        CLOUDFLARE_ACCOUNT_ID: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
      run: |
        npm install -g wrangler
        wrangler d1 migrations apply ash-website-db --config web-site/wrangler.toml

    - name: Install and test
      working-directory: web-site
      run: |
        npm ci
        npm test

    - name: Build Astro
      working-directory: web-site
      run: npm run build

    - name: Deploy to Cloudflare Pages
      env:
        CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
        CLOUDFLARE_ACCOUNT_ID: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
      run: |
        npm install -g wrangler
        wrangler pages deploy web-site/dist --project-name=green-band-aooriwu --branch main
```

### Changes from current `deploy-website`

| Current | New | Reason |
|---------|-----|--------|
| WASM → `web-site/wasm/` | WASM → `web-site/public/wasm/` | Astro serves `public/` at root; playground references `wasm/` |
| No assemble.js | `node docs/assemble.js` runs | Produces reference fragment for Astro build |
| No D1 migration | `wrangler d1 migrations apply` | Blog + feedback tables must exist |
| No npm install/test/build | `npm ci && npm test && npm run build` | Astro project needs build step |
| Deploy `web-site/` | Deploy `web-site/dist/` | Astro outputs to `dist/` |

### `build-and-test` Job

Add the reference fragment path to the existing `git add` step. Current lines 32–36:

```yaml
git add "${{ github.workspace }}/README.md" \
    "${{ github.workspace }}/npm-package/README.md" \
    "${{ github.workspace }}/ash-vscode/README.md" \
    "${{ github.workspace }}/web-site/reference.html" \
    "${{ github.workspace }}/docs/website-reference.md"
```

Change to:

```yaml
git add "${{ github.workspace }}/README.md" \
    "${{ github.workspace }}/npm-package/README.md" \
    "${{ github.workspace }}/ash-vscode/README.md" \
    "${{ github.workspace }}/web-site/reference.html" \
    "${{ github.workspace }}/web-site/src/content/reference-content.html" \
    "${{ github.workspace }}/docs/website-reference.md"
```

### Other Jobs

All other jobs (`build-and-test` main logic, `release`, `github-release`, `npm-publish`) are **unchanged**. Only the two CI changes above are needed.

## Acceptance Criteria

1. `deploy-website` job builds WASM to `web-site/public/wasm/` (not `web-site/wasm/`).
2. `deploy-website` job runs `node docs/assemble.js` to generate the reference fragment.
3. `deploy-website` job applies D1 migrations via `wrangler d1 migrations apply` before building.
4. `deploy-website` job runs `npm ci && npm test && npm run build` in `web-site/`.
5. `deploy-website` job deploys `web-site/dist/` (not `web-site/`) to Cloudflare Pages `green-band-aooriwu`.
6. `build-and-test` job's `git add` includes `web-site/src/content/reference-content.html`.
7. No other CI jobs are modified.
8. D1 migration step uses `secrets.CLOUDFLARE_API_TOKEN` and `secrets.CLOUDFLARE_ACCOUNT_ID` (assumed to exist as repository secrets).

## Implementation Hints

- The CI file is at `.github/workflows/ci.yml`.
- The current `deploy-website` job is at lines 123–149. Replace it entirely with the new job definition above.
- The `build-and-test` `git add` block is at lines 32–36. Add one line for the fragment path.
- The WASM build step uses `cp pkg/*.wasm` and `cp pkg/*.js` — this maps all generated WASM and JS files. The `ash-wasm` crate output includes `ash_wasm_bg.wasm`, `ash_wasm.js`, and potentially `ash_wasm.d.ts`.
- The `npm install -g wrangler` step is duplicated (once for migration, once for deploy). This is intentional — the two `wrangler` commands are in separate `run` steps with different working directories, and the global install ensures availability.
- The `deploy-website` job has `needs: github-release` — this ensures the WASM source is at a tagged commit and the GitHub release is created before deployment.
- D1 migration credentials: `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` must exist as repository secrets. The task assumes they are already configured (they are used by the current `deploy-website` job's `wrangler pages deploy`).
- The `wrangler.toml` file at `web-site/wrangler.toml` must have the correct committed `database_id` for the remote D1 database. This was populated in Phase 01 as a one-time manual step.
