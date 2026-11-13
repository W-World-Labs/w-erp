# Third-party notices

Project-owned W-ERP source is licensed under the MIT License in `LICENSE`.
This grant applies only to W-owned material; third-party licenses and uncertain
external assets retain the separate terms and review holds below.

## Bundled materials

- `source/opc/backend/assets/fonts/NotoSansSC-*.ttf`: existing derivatives whose
  metadata names Adobe (2014-2021), Reserved Font Name `Source`, version 2.004-H2.
  SIL OFL 1.1 is preserved in `licenses/Noto-OFL-1.1.txt`; upstream:
  https://github.com/notofonts/noto-cjk/blob/main/Sans/LICENSE . Exact conversion
  provenance was not recorded; this is disclosed as a provenance limitation.
  OFL permits modified versions and subsetting, and the declared Reserved Font
  Name is not used as the primary name of these derivatives.
- `source/opc/frontend/src/components/ui`: shadcn/ui-derived UI components.
  Copyright (c) 2023 shadcn; MIT text in `licenses/shadcn-MIT.txt`.
  https://github.com/shadcn-ui/ui/blob/main/LICENSE.md .
- `tests/ocr/fixtures/azure-*`: Microsoft sample invoices, fixed source commit
  `8ac994c2712f3104d00ca4224b0ded6fd1ecbf70`, MIT text in
  `tests/ocr/licenses/Microsoft-MIT.txt`.
- `tests/ocr/fixtures/{1078,1197}-receipt.jpg`: ExpressExpense.com SRD, MIT text
  in `tests/ocr/licenses/ExpressExpense-MIT.txt`. Publisher license statement:
  https://expressexpense.com/blog/free-receipt-images-ocr-machine-learning-dataset/ .
- The UK Model Services Contract is not bundled; the optional cache source contains public sector information
  licensed under the Open Government Licence v3.0. Publisher source and license:
  https://www.gov.uk/government/publications/model-services-contract-terms and
  https://www.nationalarchives.gov.uk/doc/open-government-licence/version/3/ .
  Government logos are excluded from that license; the whole PDF is isolated from this distribution. No endorsement is implied.
- Four Chinese OCR images and contract cases are described by the source project
  as synthetic, without real accounting or legal effect. See their manifests.
- The Chinese contract PDF with unknown redistribution rights remains outside the
  repository in an optional operator cache; its source metadata is not a license.

The unused Monotype Arial fonts and unprovenance favicon/placeholder image are
excluded. Three legacy architecture raster images are also excluded because their
creation source and redistribution rights are unrecorded. The project-owned
Mermaid text in `source/opc/docs/architecture/` is preserved; it is not claimed as
the source of those raster images.

## Dependency boundary

Rust and npm libraries remain under their own package licenses. The dependency
inventory records package metadata, versions, sources and license fields when
available; it is not a legal opinion or a completed vulnerability audit.
Public CI installs public dependencies and validates source on a standard hosted
runner. It publishes no compiled artifacts. Binary and container distributions
require their own dependency-license and notice packaging acceptance; source
checks do not establish production activation or acceptance.

## Public W browser SDKs

`vendor/w-http-web` and `vendor/xradar-js` contain only the corresponding public
W-Radar SDK directories pinned in each `SOURCE.json`. Their MIT license is
retained alongside the code. Runtime APIs and checked-in distribution entrypoints
are preserved; package scripts and development dependencies are omitted.
These are owned by W-Radar, not W-Commons. No application business logic is copied.
