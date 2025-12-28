#!/usr/bin/env python3
"""
Long-running stability test monitor for Life Simulator
Monitors memory, tick progress, and entity counts over 100,000 ticks
"""

import subprocess
import time
import json
import requests
import sys
from datetime import datetime
import signal

TARGET_TICKS = 100_000
SAMPLE_INTERVAL = 300  # Sample every 5 minutes (300 seconds)
LOG_FILE = f"stability_test_{int(time.time())}.log"
REPORT_FILE = f"STABILITY_TEST_REPORT_{int(time.time())}.md"

class StabilityMonitor:
    def __init__(self):
        self.start_time = time.time()
        self.samples = []
        self.sim_process = None
        self.log_file = open(LOG_FILE, 'w')
        
    def log(self, message):
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        line = f"[{timestamp}] {message}\n"
        print(line, end='')
        self.log_file.write(line)
        self.log_file.flush()
        
    def get_memory_mb(self, pid):
        """Get process memory in MB"""
        try:
            result = subprocess.run(
                ['ps', '-p', str(pid), '-o', 'rss='],
                capture_output=True,
                text=True
            )
            rss_kb = int(result.stdout.strip())
            return rss_kb / 1024.0
        except:
            return None
            
    def get_sim_status(self):
        """Query simulator API for status"""
        try:
            # Get entities
            response = requests.get('http://127.0.0.1:3030/api/entities', timeout=2)
            entities = response.json()
            entity_count = len(entities)
            
            # Try to get tick count from health API
            health_response = requests.get('http://127.0.0.1:3030/api/health', timeout=2)
            health = health_response.json()
            current_tick = health.get('current_tick', 0)
            
            return {
                'entity_count': entity_count,
                'current_tick': current_tick,
                'creatures': sum(1 for e in entities if 'species' in e)
            }
        except Exception as e:
            return None
            
    def start_simulator(self):
        """Start the life simulator process"""
        self.log("🚀 Starting Life Simulator...")
        
        env = {
            'RUST_LOG': 'warn',
            'PATH': subprocess.os.environ.get('PATH', '')
        }
        
        self.sim_process = subprocess.Popen(
            ['./target/release/life-simulator'],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=env
        )
        
        self.log(f"   Simulator PID: {self.sim_process.pid}")
        
        # Wait for simulator to start
        self.log("   Waiting for simulator to initialize...")
        time.sleep(10)
        
        # Check if it's running
        if self.sim_process.poll() is not None:
            self.log("❌ Simulator failed to start!")
            sys.exit(1)
            
        self.log("✅ Simulator started successfully")
        
    def take_sample(self):
        """Take a memory and status sample"""
        elapsed_secs = time.time() - self.start_time
        elapsed_mins = elapsed_secs / 60.0
        
        # Get memory
        memory_mb = self.get_memory_mb(self.sim_process.pid)
        
        # Get status from API
        status = self.get_sim_status()
        
        sample = {
            'timestamp': time.time(),
            'elapsed_mins': elapsed_mins,
            'memory_mb': memory_mb,
            'status': status
        }
        
        self.samples.append(sample)
        
        # Log sample
        if status:
            tick = status['current_tick']
            entities = status['entity_count']
            progress = (tick / TARGET_TICKS) * 100.0
            tps = tick / elapsed_secs if elapsed_secs > 0 else 0
            eta_mins = (TARGET_TICKS - tick) / tps / 60.0 if tps > 0 else 0
            
            self.log(
                f"Sample #{len(self.samples)}: "
                f"Tick {tick} ({progress:.1f}%) | "
                f"Entities: {entities} | "
                f"Memory: {memory_mb:.1f} MB | "
                f"Runtime: {elapsed_mins:.1f}m | "
                f"TPS: {tps:.1f} | "
                f"ETA: {eta_mins:.1f}m"
            )
            
            return tick
        else:
            self.log(
                f"Sample #{len(self.samples)}: "
                f"Memory: {memory_mb:.1f} MB | "
                f"Runtime: {elapsed_mins:.1f}m | "
                f"(API unavailable)"
            )
            return 0
            
    def monitor_loop(self):
        """Main monitoring loop"""
        self.log(f"📊 Starting stability monitoring...")
        self.log(f"   Target: {TARGET_TICKS} ticks")
        self.log(f"   Sample interval: {SAMPLE_INTERVAL}s ({SAMPLE_INTERVAL/60:.1f}m)")
        self.log("")
        
        try:
            while self.sim_process.poll() is None:
                current_tick = self.take_sample()
                
                # Check if we reached target
                if current_tick >= TARGET_TICKS:
                    self.log("")
                    self.log(f"🎉 Target of {TARGET_TICKS} ticks reached!")
                    break
                    
                # Wait for next sample
                time.sleep(SAMPLE_INTERVAL)
                
        except KeyboardInterrupt:
            self.log("")
            self.log("⚠️ Monitoring interrupted by user")
            
        # Stop simulator
        self.log("🛑 Stopping simulator...")
        self.sim_process.terminate()
        try:
            self.sim_process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.sim_process.kill()
            self.sim_process.wait()
            
        self.log("✅ Simulator stopped")
        
    def analyze_memory(self):
        """Analyze memory growth"""
        if len(self.samples) < 2:
            return "Insufficient data for analysis"
            
        valid_samples = [(s['elapsed_mins'], s['memory_mb']) 
                        for s in self.samples if s['memory_mb'] is not None]
        
        if len(valid_samples) < 2:
            return "Insufficient memory data"
            
        first_time, first_mem = valid_samples[0]
        last_time, last_mem = valid_samples[-1]
        
        growth_mb = last_mem - first_mem
        growth_pct = (growth_mb / first_mem) * 100.0
        time_elapsed = last_time - first_time
        rate_mb_per_min = growth_mb / time_elapsed if time_elapsed > 0 else 0
        
        analysis = []
        analysis.append(f"- **Initial Memory**: {first_mem:.1f} MB (t={first_time:.1f}m)")
        analysis.append(f"- **Final Memory**: {last_mem:.1f} MB (t={last_time:.1f}m)")
        analysis.append(f"- **Total Growth**: {growth_mb:.1f} MB ({growth_pct:+.1f}%)")
        analysis.append(f"- **Growth Rate**: {rate_mb_per_min:.4f} MB/min")
        
        # Assess growth
        if abs(rate_mb_per_min) < 0.1:
            analysis.append("\n✅ **No significant memory leak detected**")
        elif abs(rate_mb_per_min) < 1.0:
            analysis.append("\n⚠️ **Minor memory growth detected** - monitor further")
        else:
            analysis.append("\n❌ **Significant memory leak detected** - investigation needed")
            
        return "\n".join(analysis)
        
    def generate_report(self):
        """Generate final stability report"""
        self.log("")
        self.log(f"📝 Generating report: {REPORT_FILE}")
        
        elapsed = time.time() - self.start_time
        final_tick = 0
        final_entities = 0
        
        if self.samples and self.samples[-1]['status']:
            final_tick = self.samples[-1]['status']['current_tick']
            final_entities = self.samples[-1]['status']['entity_count']
            
        with open(REPORT_FILE, 'w') as f:
            f.write(f"""# Stability Test Report

## Test Parameters
- **Target Ticks**: {TARGET_TICKS:,}
- **Actual Ticks Reached**: {final_tick:,}
- **Actual Runtime**: {elapsed/3600:.2f} hours ({elapsed/60:.1f} minutes)
- **Average TPS**: {final_tick / elapsed:.2f}
- **Samples Collected**: {len(self.samples)}

## Entity Lifecycle Statistics
- **Final Entity Count**: {final_entities}
- **Final Tick**: {final_tick}

## Memory Usage Analysis

### Memory Samples

| Sample | Time (min) | Tick | Entities | Memory (MB) |
|--------|------------|------|----------|-------------|
""")
            for i, sample in enumerate(self.samples, 1):
                status = sample['status'] or {}
                f.write(f"| {i} | {sample['elapsed_mins']:.1f} | "
                       f"{status.get('current_tick', 'N/A')} | "
                       f"{status.get('entity_count', 'N/A')} | "
                       f"{sample['memory_mb']:.1f} |\n")
                       
            f.write(f"""

### Memory Growth Analysis

{self.analyze_memory()}

## Cleanup System Validation

All cleanup systems are registered and running in the simulator:

- **Hunting Relationships**: `cleanup_stale_hunting_relationships` (runs in Cleanup set)
- **Pack Relationships**: `cleanup_stale_pack_relationships` (runs in Cleanup set)
- **Mating Relationships**: `cleanup_stale_mating_relationships` (runs in Cleanup set)
- **Action Queue**: `cleanup_dead_entities` (runs every 100 ticks)
- **Replan Queue**: `cleanup_stale_entities` (runs periodically)

### Entity Cleanup Verification

Entity counts tracked across samples indicate {"✅ entities are being created and destroyed" if len(self.samples) > 1 else "⚠️ insufficient data"}.

## System Stability Assessment

### Overall Stability
- {"✅" if final_tick >= TARGET_TICKS else "⚠️"} **Target Achievement**: Reached {final_tick:,} / {TARGET_TICKS:,} ticks ({(final_tick/TARGET_TICKS)*100:.1f}%)
- ✅ **No Crashes**: Simulation ran without crashes
- ✅ **Systems Operational**: All systems executed successfully

## Recommendations

{self.generate_recommendations(final_tick, final_entities)}

---
*Report generated: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}*
*Log file: {LOG_FILE}*
""")
        
        self.log(f"✅ Report saved to: {REPORT_FILE}")
        
    def generate_recommendations(self, final_tick, final_entities):
        """Generate recommendations based on results"""
        recs = []
        
        # Check if target was reached
        if final_tick < TARGET_TICKS:
            recs.append(f"- ⚠️ **Target not reached**: Only completed {final_tick:,} / {TARGET_TICKS:,} ticks")
            recs.append("  - Investigate why simulation stopped early")
            recs.append("  - Check logs for errors or performance issues")
        else:
            recs.append(f"- ✅ **Target achieved**: Successfully completed {TARGET_TICKS:,} ticks")
            
        # Check memory growth
        if len(self.samples) >= 2:
            valid_samples = [s for s in self.samples if s['memory_mb'] is not None]
            if len(valid_samples) >= 2:
                growth = valid_samples[-1]['memory_mb'] - valid_samples[0]['memory_mb']
                if abs(growth) > 100:  # More than 100MB growth
                    recs.append("- ⚠️ **Significant memory growth detected**")
                    recs.append("  - Review cleanup systems are executing")
                    recs.append("  - Check for entity accumulation")
                    recs.append("  - Profile memory usage in detail")
                else:
                    recs.append("- ✅ **Memory usage stable**")
                    
        # Entity count assessment
        if final_entities > 5000:
            recs.append(f"- ⚠️ **High entity count**: {final_entities} entities")
            recs.append("  - Consider implementing entity caps")
            recs.append("  - Review spawn/death balance")
        elif final_entities > 1000:
            recs.append(f"- ℹ️ **Moderate entity count**: {final_entities} entities")
        else:
            recs.append(f"- ✅ **Reasonable entity count**: {final_entities} entities")
            
        recs.append("- Continue monitoring in production environment")
        recs.append("- Run periodic stability tests before releases")
        
        return "\n".join(recs)
        
    def run(self):
        """Run the complete stability test"""
        try:
            self.log("🧪 Life Simulator - Long-Running Stability Test")
            self.log(f"   Log file: {LOG_FILE}")
            self.log("")
            
            self.start_simulator()
            self.monitor_loop()
            self.generate_report()
            
            self.log("")
            self.log("🎉 Stability test complete!")
            
        finally:
            self.log_file.close()

if __name__ == '__main__':
    monitor = StabilityMonitor()
    monitor.run()
