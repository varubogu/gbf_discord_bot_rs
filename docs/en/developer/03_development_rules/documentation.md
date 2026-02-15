# Documentation Principles

## Purpose

This document declares what development documentation should be.
Its purpose is not to list procedures or checklists, but to clarify the values and stance that form the basis of decisions.

## Stance

Documentation is not an appendix to implementation; it is an official deliverable that shares design intent and operational decisions.
Reduce reader-dependent interpretation and maintain a state where the whole team stands on the same assumptions.

## Accuracy

Documentation must always match the current specification.
Even if historical context or temporary decisions remain, prioritize making the current correct state clearly readable.

## Clarity

Avoid ambiguity; wording should communicate intent, constraints, and responsibility boundaries.
Maintain enough density that readers can make decisions without guessing.

## Consistency

Use the same terms for the same concepts and keep meanings stable across documents.
Prefer consistent interpretation over expressive freedom.

## Traceability

Changes should come with reasons; keep a state where you can explain later why something changed.
Prefer preserving decision context, not just recording outcomes.

## Maintainability

Documentation should be maintained with future changes in mind.
Treat it not as “write once”, but as an operational asset that is continuously updated.

## Japanese as the canonical source

At present, the Japanese documentation is the baseline and treated as the canonical source.
Write with future multilingual expansion in mind so that meanings don’t collapse when translated.

## Principles for maintaining Japanese/English docs

This project aims to maintain both Japanese and English versions of development documentation.
Even if the language differs, specifications, constraints, and decision criteria must match; do not normalize inconsistencies.

Even if Japanese is updated first for some period, do not treat maintaining the English version as an optional future task.
The English version should not be a summary of Japanese; it should be maintained as a peer document that can share equivalent design decisions.

## Link style

Use relative paths for links so they resolve within the repository.
Do not use machine-specific absolute paths because they won’t resolve when shared or on GitHub.

## Relationship to design

Documentation is not only to constrain implementation; it should function as shared knowledge that supports design decisions.
Architectural separation of concerns, boundaries, and constraints should be expressed so that contributors can reach the same understanding regardless of experience level.

## Trustworthiness

Keeping documentation trustworthy is a prerequisite for development speed and quality.
Prioritize being a document that is referenced when making decisions, not one that is ignored.
