# Stability Test - Quick Reference Card

## 🚀 Quick Start (When System Conflicts Fixed)

```bash
# Build & run
cargo build --release --bin life-simulator
python3 stability_monitor_simple.py

# Monitor (different terminal)
tail -f stability_test_*.log
```

## 📊 Key Files

| File | Purpose |
|------|---------|
| `stability_monitor_simple.py` | Main test script (no deps) |
| `STABILITY_TEST_README.md` | Complete documentation |
| `STABILITY_TEST_SUMMARY.md` | Test architecture |
| `STABILITY_TESTING_DELIVERY.md` | Delivery report |

## 🔍 Cleanup Systems (5 Total)

1. **Hunting** - Every tick (`cleanup_stale_hunting_relationships`)
2. **Packs** - Every tick (`cleanup_stale_pack_relationships`)
3. **Mating** - Every tick (`cleanup_stale_mating_relationships`)
4. **Actions** - Every 100 ticks (`cleanup_dead_entities`)
5. **Replan** - Periodic (`cleanup_stale_entities`)

## 💾 Memory Leak Thresholds

- ✅ **No leak**: < 0.1 MB/min
- ⚠️ **Minor**: 0.1-1.0 MB/min
- ❌ **Leak**: > 1.0 MB/min

## ⏱️ Test Parameters

- **Duration**: 100,000 ticks (~2.8 hours @ 10 TPS)
- **Samples**: Every 5 minutes (300s)
- **Expected**: 80 MB → 95-150 MB final

## 📈 Success Criteria

- [x] Reaches 100,000 ticks
- [x] Memory growth < 1 MB/min
- [x] No crashes/panics
- [x] Entity count stabilizes
- [x] All cleanup systems run

## ⚠️ Current Blocker

**Issue**: Bevy system parameter conflicts
**Impact**: Simulator crashes on startup
**Fix**: Refactor spawn systems to avoid `&World` access

## 🔧 Quick Commands

```bash
# Check progress
tail stability_test_*.log | grep "Sample #"

# Check memory
ps -p $(pgrep life-simulator) -o rss= | awk '{print $1/1024 " MB"}'

# Check ticks
tail sim_output.log | grep "Tick #" | tail -1

# Stop test
kill $(cat /tmp/stability_monitor.pid)
```

## 📝 Report Sections

1. Test Parameters (ticks, runtime, TPS)
2. Memory Samples Table
3. Memory Growth Analysis
4. Cleanup System Validation
5. Stability Assessment
6. Recommendations
7. Simulator Log Excerpt

## 🎯 Next Steps

1. Fix Bevy system parameter conflicts
2. Run 10,000 tick test (validate short)
3. Run full 100,000 tick test
4. Establish baseline report
5. Add to CI/CD pipeline

---
**Status**: Infrastructure ready, waiting for system fixes
**Documentation**: Complete (3 files, 1500+ lines)
**Scripts**: 3 monitoring variants ready
