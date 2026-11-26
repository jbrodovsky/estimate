## Summary

<!-- What does this change, and why? Link any related issue (for example "Closes #123"). -->

## Checklist

- [ ] `just ci` passes locally.
- [ ] New or changed public items follow the doc-comment template in AGENTS.md (`just doc-lint`).
- [ ] No panics, `unsafe` or call-site `#[allow]` in library code.
- [ ] CHANGELOG.md has an entry under `## [Unreleased]`, if the change is user-visible.
- [ ] docs/design.md is updated, if a public trait changed.
- [ ] New filters or manifolds come with consistency tests (analytic ground truth, NEES/NIS,
      or `proptest` axioms).
