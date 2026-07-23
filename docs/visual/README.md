# Visual identity — sharecli

Design-system reference for audit-v38 **C10** (L96–L107).

| Doc | Purpose |
|-----|---------|
| [IDENTITY.md](IDENTITY.md) | Signature mark + what makes sharecli visually unique |
| [VISUAL_SPEC.md](VISUAL_SPEC.md) | Palette, type, states, major surfaces |
| [empty-states.md](empty-states.md) | First-run vs filtered empty copy + CTAs (L100) |
| [error-states.md](error-states.md) | Disconnect panel + tier-1 illustration (L101) |
| [loading-states.md](loading-states.md) | Dashboard skeleton rows + connecting phases (L99) |
| [PROVENANCE.md](PROVENANCE.md) | Asset provenance tiers (1/2/3) |
| [typography.md](typography.md) | Type-role scale + rationale |
| [motion.md](motion.md) | Motion tokens + prefers-reduced-motion |
| [theming.md](theming.md) | Dark / light Backbone-2 pair |
| [golden-visual-tests.md](golden-visual-tests.md) | Screenshot / visual-regression plan (L107 soft) |

**Sources of truth**

- Colors: [`assets/tokens.css`](../../assets/tokens.css) ↔ Rust mirror [`src/theme.rs`](../../src/theme.rs)
- Mark: [`assets/brand/`](../../assets/brand/)
- Contrast: [`docs/a11y/contrast.md`](../a11y/contrast.md)
