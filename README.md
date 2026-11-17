# heif2jpeg

Fast and simple HEIC/HEIF to JPEG converter for Node.js. Native performance, zero runtime dependencies.

Built with node-api.

## Install

```bash
npm install heif2jpeg
```

Prebuilt binaries are provided for:

| Platform | Architecture |
|----------|-------------|
| macOS | x64, arm64 |
| Linux (glibc) | x64, arm64 |
| Linux (musl) | x64, arm64 |
| Windows | x64, arm64 |

Works with Node.js, Bun, and Deno.

## Usage

```js
const { heifToJpeg } = require("heif2jpeg");
const fs = require("fs");

const heic = fs.readFileSync("photo.heic");
const jpeg = await heifToJpeg(heic, { quality: 85 });
fs.writeFileSync("photo.jpg", jpeg);
```

## API

### `heifToJpeg(input, options?)`

Convert a HEIF/HEIC buffer to JPEG.

- **input** `Buffer`: HEIF/HEIC file contents
- **options.quality** `number`: JPEG quality, 1-100 (default: 85)
- Returns `Promise<Buffer>`: JPEG file contents

## How it works

All processing runs on the libuv thread pool — the main thread is never blocked.

1. [libheif](https://github.com/strukturag/libheif) parses the HEIF container
2. [libde265](https://github.com/strukturag/libde265) decodes the HEVC payload to raw RGB pixels
3. [jpeg-encoder](https://crates.io/crates/jpeg-encoder) (pure Rust) encodes to JPEG

## Building from source

Requires Rust, CMake, and a C/C++ compiler.

```bash
git clone --recurse-submodules https://github.com/photon-hq/heif2jpeg.git
cd heif2jpeg
npm install
npx napi build --platform --release
npm test
```

## License

Photon's wrapper is licensed under [Apache-2.0](LICENSE). The native addons
include libheif and libde265 under LGPL-3.0-or-later. See
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for the licenses, pinned
sources and instructions for rebuilding with modified libraries.

## Releasing to npm

Pull requests compile the six native targets and dry-run the Buildspace pack;
they do not publish. A push to `main` builds the same targets and stages all
seven packages under the npm `staging` dist-tag. Buildspace computes the next
version from the existing `vX.Y.Z` tags and merged PRs. Both the staging and
production tarballs use the binaries from that run. Don't bump the manifests
or push a release tag manually.

To release, select **Actions → Promote to production → Run workflow** on
`main`. Enter the full staging version from the build summary to promote a
specific build, or leave it blank to use the current staging dist-tag. After
production approval, Buildspace publishes the stored candidates under
`latest` and creates the version tag. Promotion does not rebuild.

**Maintainer setup:** Before the first staging push, configure the GitHub
`staging` and `production` environments for `main` only, with required
reviewers for `production`. Configure an npm trusted publisher for
`photon-hq/heif2jpeg` on the wrapper and each of the six platform packages:
`build.yml` with environment `staging` before staging, and `promote.yml`
with environment `production` shortly before the first promotion. Allow
`npm publish` and `npm dist-tag` on both. See the
[Buildspace setup guide](https://github.com/photon-hq/buildspace/blob/f2b0f40bb808a721d71711a4db59d234d86736a6/README.md#package-stage-and-promote).
