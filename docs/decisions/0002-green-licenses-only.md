# 0002. Only green licenses in the shipped product

Date: 2026-09-25. Status: accepted.

## Decision

Every component of the shipped app (code, libraries, codecs, runtimes,
models, fonts, services) must be green: permissive, with obligations we
can meet by notices alone, and no field-of-use restriction. A yellow or
red item is either resolved or replaced; if no green replacement
exists, we build our own.

## Consequences

- HTDemucs (scientific-use-only weights) is excluded from the product.
- Codecs come from the operating system where patent pools apply.
- Every model ships with a model card: source, license, conversion
  steps, parity record.
