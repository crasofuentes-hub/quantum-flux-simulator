# Reproduce command

`flux-sim reproduce` is a single reproducible entry point over the current effective analysis pipeline.

## Current behavior

- if `input_path` is a file, it runs the existing single-file analysis flow,
- if `input_path` is a directory, it runs the existing batch flow,
- it writes a JSON output report,
- it writes a separate reproducibility manifest.

## Manifest fields

The manifest records:

- manifest schema version,
- tool version,
- mode (`analyze` or `batch`),
- seed,
- effective runtime parameters,
- analysis/report schema versions,
- input path and kind,
- deterministic input fingerprint,
- generated output paths.

## Important limitation

The fingerprint is currently a deterministic **FNV-1a 64-bit content fingerprint** over the input file or recursively over directory contents.
It is suitable as a lightweight reproducibility marker inside this prototype.

It is **not** presented as a cryptographic integrity proof.