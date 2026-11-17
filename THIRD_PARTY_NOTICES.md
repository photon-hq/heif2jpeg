# Native decoder notices

Photon's heif2jpeg wrapper is licensed under [Apache-2.0](LICENSE). The
prebuilt native addons statically link the following libraries. Both libraries
and their use in these addons are covered by the **GNU Lesser General Public
License, version 3 or later**. Copies of the [LGPLv3](LICENSES/LGPL-3.0.txt)
and [GPLv3](LICENSES/GPL-3.0.txt) accompany this package.

| Library | Copyright notices | Corresponding library source |
| --- | --- | --- |
| libheif 1.21.2 | Copyright (c) 2017–2020 Struktur AG; Copyright (c) 2017–2026 Dirk Farin | [Source and upstream notices](https://github.com/strukturag/libheif/tree/f283763317d8b7fd98d0c3f8bbb6eac588e6b531) |
| libde265 1.0.18 | Copyright (c) 2013–2014 Struktur AG; Copyright (c) 2013–2026 Dirk Farin | [Source and upstream notices](https://github.com/strukturag/libde265/tree/2850f79ac7923b4a0498f2f123653f64dde628f4) |

## Rebuild with a modified decoder

The [public heif2jpeg repository](https://github.com/photon-hq/heif2jpeg)
contains the addon source and build scripts. Each release pins its decoders as
Git submodules. Clone the repository, check out the commit identified in your
packaged notice, and run:

```sh
git submodule update --init --recursive
npm ci
```

Make your changes in `deps/libheif` or `deps/libde265`. With Rust, CMake,
a C/C++ compiler and Node.js installed, run `npm run build` and `npm test`.
The build produces a replacement `heif2jpeg.<platform>.node`. Place it beside
the wrapper's `index.js` to load it instead of the prebuilt optional package.
