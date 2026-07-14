# ADR 0004 — No first-party MCP server in sharecli

**Status:** Accepted  
**Date:** 2026-07-14  
**Deciders:** sharecli maintainers  
**Traceability:** audit-v38 C06 L57 (MCP server provenance)

## Context

Cluster C06 scores **MCP server provenance** (phenoMCP pin / attested MCP
servers). sharecli is a **local process supervisor / CLI daemon**. It does not
expose an MCP tool surface and does not vendor a phenoMCP server binary.

## Decision

1. **Out of scope:** First-party MCP server packaging, pin manifests, and
   MCP attestation for this repository.
2. **In scope elsewhere:** If Phenotype agents need MCP, pin and attest those
   servers in the MCP-owning repos (e.g. thegent / agileplus-mcp-intent).
3. **Revisit** only if sharecli ships an MCP transport or claims MCP tools.

## Consequences

- Auditors should score L57 as **N/A / seeded** via this ADR, not as a missing
  product gap, until scope expands.
- Supply-chain lifts for sharecli remain crates.io pins, SLSA, cosign soft,
  DCO, and license scan — not MCP pin manifests.

## References

- Rubric: `audit/rubric/audit-30-pillar/audit-30-pillar-L51-L60-supply-chain.md`
- Related: `docs/adr/0002-eval-surface-out-of-scope.md` (same profile discipline)
