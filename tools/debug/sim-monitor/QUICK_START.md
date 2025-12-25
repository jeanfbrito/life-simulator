# sim-monitor Quick Start

## Installation

```bash
# From project root
cd /Users/jean/Github/life-simulator

# Build release version
cargo build --release --package sim-monitor
```

## Usage

### Start monitoring (default settings)
```bash
cargo run --package sim-monitor
```

Or use the release binary:
```bash
./target/release/sim-monitor
```

### Custom configuration
```bash
# Different port
./target/release/sim-monitor --url http://localhost:8080

# Slower refresh (2 seconds)
./target/release/sim-monitor --refresh 2

# Both options
./target/release/sim-monitor --url http://localhost:8080 --refresh 2
```

## Keyboard Controls

| Key | Action |
|-----|--------|
| `q` | Quit |
| `Esc` | Quit |
| `r` | Force refresh |

## Requirements

1. **Running Simulator**: Life Simulator must be running with debug API enabled
   ```bash
   cargo run --bin life-simulator
   ```

2. **Default Port**: Simulator should be on http://127.0.0.1:54321
   - Or specify custom URL with `--url`

## Display

```
┌────────────────────────────────────────────┐
│ Life Simulator Monitor | TPS: 59.8 | OK   │
├──────────────┬─────────────────────────────┤
│ ENTITIES     │ HEALTH STATUS              │
│  Deer: 12    │  ✓ Overall: ok             │
│  Fox: 6      │  ✓ TPS: 59.8 (Excellent)   │
│  Rabbit: 24  │  ⚠ alerts: 0               │
├──────────────┴─────────────────────────────┤
│ RECENT ALERTS                              │
│  No alerts                                 │
└────────────────────────────────────────────┘
```

## Troubleshooting

### "Disconnected" status
- Check simulator is running: `ps aux | grep life-simulator`
- Verify port matches: default is 54321
- Check firewall isn't blocking localhost

### No entities showing
- Simulator may have just started (entities spawn over time)
- Check `/api/entities` directly: `curl http://localhost:54321/api/entities`

### Slow updates
- Increase refresh interval: `--refresh 2`
- Check simulator TPS (shown in header)

## Development

### Run tests
```bash
cargo test --package sim-monitor
```

### Build debug version
```bash
cargo build --package sim-monitor
./target/debug/sim-monitor
```

## Performance

- **CPU**: < 0.5% during normal operation
- **Memory**: < 10 MB
- **Network**: ~5 KB/sec at 1Hz refresh

## More Information

- **README.md** - Full documentation
- **ARCHITECTURE.md** - Technical design
- **TEST_REPORT.md** - Test coverage details
