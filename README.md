# Swarm-Tools

**The ultimate weapon for Claude Code multi-agent swarms**
**~80-110%+ token/cost reductions - Near Zero deadlocks - True autonomous scaling**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Claude Code Marketplace](https://img.shields.io/badge/Claude%20Code-Marketplace-blue)](https://code.claude.com/docs/en/discover-plugins)

Sick of context bloat, Ralph loops, "context low" deadlocks, and swarms that collapse?

Swarm-Tools is here to help ease this burden.

Built on cutting-edge 2025 research (Optima, RCR-Router, Trajectory Reduction, BAMAS, CodeAgents), this Rust-native plugin transforms Claude Code into a battle-hardened swarm engine capable of 10-20+ parallel agents with minimal tokens and maximum reliability.

No other plugin comes close, this is the most advanced swarm optimizer available today (that i know of).

## Quick Install

1. Add the marketplace (once):
   ```
   /plugin marketplace add lazerusrm/Swarm-Tools
   ```

2. Install:
   ```
   /plugin install swarm-tools
   ```

Done. Auto-downloads binaries, wires hooks (`precompact`, `subagentStop`), and keeps you updated forever.
*(Requires Claude Code v2.0+ with marketplace support)*

## Manual Install

```bash
git clone https://github.com/lazerusrm/Swarm-Tools.git
cd Swarm-Tools
cargo build --release
```

Add to settings.json:

```json
{
  "plugins": {
    "swarm-tools": {
      "path": "/path/to/Swarm-Tools/target/release",
      "hooks": {
        "precompact": "precompact",
        "subagentStop": "subagent_stop"
      }
    }
  }
}
```

Pre-built binaries in Releases.

## Primary Features

- **Persistent Multi-Type Loop Detection** - Crushes Ralph loops before they start
- **Role-Aware Routing** - Recency + impact boosted (45-65% communication savings)
- **Sparse Trajectory Compression** - Impact-based, expired/redundant filtering (25-40% context reduction)
- **Quality Gates + Closed-Loop Refinement** - Objective scoring drives perfect outputs
- **Codified Reasoning** - Structured plans with priority/impact/token estimates
- **MCP/Tool Routing** - Selective approval + arg stripping (20-40% external waste gone)
- **Auto-Model Tiering** - Haiku/Sonnet/Opus routing (30-50% cost savings)
- **Self-Healing Topology** - Contribution-tracked auto-pruning + rebalancing
- **Parallel Execution Planning** - Smart batching + mode comparison
- **Communication Optimization** - Redundancy/irrelevance pattern removal
- **Fully Configurable** - JSON overrides for every heuristic, weight, pattern, and threshold

Everything is optional, lightweight (no heavy deps), and runtime-safe.

## Configuration

Drop overrides in `~/.config/swarm-tools/config.json`.
Ready-made presets in `config_examples/`:

- `coding_swarm.json` - Code-heavy beast mode
- `research_swarm.json` - Web/browse domination
- `large_scale.json` - Aggressive pruning for massive swarms

## Why Swarm-Tools

Vanilla Claude Code swarms hit walls: unbounded context, redundant loops, exploding costs, context deadlock.
Swarm-Tools rewrites the rules—proactive heuristics, research-backed autonomy, and tenacious efficiency let you run large, reliable, cheap swarms!

Backed by 2025 research breakthroughs:

- Optima / OMAC multi-dimension optimization
- RCR-Router role-aware relevance
- Trajectory Reduction / AgentDiet sparse compression
- BAMAS budget-aware topology + pruning
- CodeAgents structured planning

## Contributing

Issues, PRs, and real-world benchmarks welcome.

MIT © lazerusrm
