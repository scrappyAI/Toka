# Rule Library

This directory contains a **portable library of engineering rules**.  Each file is written in the same XML‐flavoured markup and represents a standalone, composable `ProjectRule`.

## Directory layout

| Path | Purpose |
|------|---------|
| `00_baseline.mdc` | Core rules that apply to every workspace unless explicitly overridden. |
| `agent-*` / `security-*` | Security‐focused guidance for autonomous agents and hardening. |
| `doc-*` | Documentation generation & maintenance. |
| `test-*` | Testing & coverage guidelines. |
| `refactor-*` | Structured refactoring playbooks. |
| `*.mdc` | Domain‐specific rules (see header metadata for description & category). |

> **Note**: Files are prefixed with a two–digit ordinal (`00`, `10`, `20` …) to make the evaluation order explicit.

## Composing rules

Rules support an `<Extends>` tag that allows incremental composition.  For example:

```xml
<ProjectRule name="MyMicroserviceRules">
  <Extends>Baseline, HardenSecuritySurfaceAgents, EnhanceTestCoverage</Extends>

  <Description>
  Hardening & quality rules tailored for the Foo microservice.
  </Description>
</ProjectRule>
```

* `Baseline` provides the universal foundations.
* `HardenSecuritySurfaceAgents` layers additional security constraints.
* `EnhanceTestCoverage` enforces higher bar on code coverage.

Later rules MAY override sections from earlier ones where necessary.

## Authoring new rules

1. **Name** the file descriptively and give it a numeric prefix if ordering matters.
2. **Describe** the intent & scope in `<Description>`.
3. **Declare** whether the rule applies automatically (`alwaysApply: true`) or must be opted into.
4. **Compose** using `<Extends>` and/or `<Includes>` rather than duplicating content.
5. **Version** substantive changes with semantic versioning comments in the header.

See `00_baseline.mdc` for a full example.