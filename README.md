# Swarm-Tools

**The most advanced optimization plugin for Claude Code multi-agent swarms**
*Token-efficient, deadlock-resistant, autonomous swarm governance — 80-110%+ efficiency gains*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Claude Code Plugin](https://img.shields.io/badge/Claude%20Code-Marketplace-blue)](https://code.claude.com/docs/en/discover-plugins)

## Quick Install (Recommended)

1. Add the marketplace (once):
   ```
   /plugin marketplace add lazerusrm/Swarm-Tools
   ```

2. Install the plugin:
   ```
   /plugin install swarm-tools
   ```

That's it—auto-downloads binaries, sets up hooks (`precompact`, `subagentStop`), and enables updates.

(Requires Claude Code v2.0+ with marketplace support.)

## Manual Install (Advanced / Offline)

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

Pre-built binaries available in Releases (macOS/Linux).

## Features

- Persistent multi-type loop detection
- Role-aware routing (recency + impact, 45-65% savings)
- Sparse trajectory compression (25-40%)
- Quality gates + iterative refinement
- Codified reasoning with estimates
- MCP/tool routing
- Auto-model tiering (Haiku/Sonnet/Opus)
- Self-healing topology (pruning/rebalancing)
- Parallel planning + communication optimization
- Fully configurable JSON overrides

## Configuration

Drop overrides in `~/.config/swarm-tools/config.json`. Examples in `config_examples/`:

- `coding_swarm.json`
- `research_swarm.json`
- `large_scale.json`

## Why Swarm-Tools?

Solves shared-context bloat/deadlocks at scale, inspired by 2025 research (Optima, RCR-Router, Trajectory Reduction, BAMAS, CodeAgents).

## Contributing

Issues/PRs welcome—especially benchmarks!

MIT © lazerusrm
