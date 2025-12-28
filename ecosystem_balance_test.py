#!/usr/bin/env python3
"""
Ecosystem Balance Test - 10,000 Tick Simulation
Monitors all species populations via API and validates ecosystem stability
"""

import subprocess
import time
import json
import sys
import os
from datetime import datetime
from collections import defaultdict
from urllib.request import urlopen
from urllib.error import URLError

TARGET_TICKS = 10_000
SAMPLE_INTERVAL_TICKS = 500  # Sample every 500 ticks
API_BASE = "http://127.0.0.1:54321"
POLL_INTERVAL = 2  # Poll API every 2 seconds

REPORT_FILE = f"ECOSYSTEM_BALANCE_RESULTS.md"
DATA_FILE = f"ecosystem_test_results_{datetime.now().strftime('%Y%m%d_%H%M%S')}/population_data.json"
SIM_LOG = "ecosystem_sim.log"

class EcosystemBalanceTest:
    def __init__(self):
        self.start_time = time.time()
        self.sim_process = None
        self.population_history = []
        self.current_tick = 0
        self.last_sampled_tick = 0
        self.starting_tick = None  # Will be set when simulator starts

        # Create results directory
        os.makedirs(os.path.dirname(DATA_FILE), exist_ok=True)

    def log(self, message):
        timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        line = f"[{timestamp}] {message}"
        print(line)
        sys.stdout.flush()

    def get_api_data(self, endpoint):
        """Fetch data from API endpoint"""
        try:
            with urlopen(f"{API_BASE}{endpoint}", timeout=5) as response:
                if response.status == 200:
                    return json.loads(response.read().decode('utf-8'))
            return None
        except (URLError, Exception):
            # Don't spam errors if simulator not ready
            return None

    def get_population_counts(self):
        """Get current population counts for all species"""
        entities = self.get_api_data("/api/entities")
        if not entities or 'entities' not in entities:
            return None

        counts = defaultdict(int)
        for entity in entities['entities']:
            species = entity.get('species', 'unknown')
            counts[species] += 1

        return dict(counts)

    def get_current_tick(self):
        """Get current simulation tick from world info"""
        world_info = self.get_api_data("/api/world_info")
        if world_info and 'current_tick' in world_info:
            return world_info['current_tick']

        # Fallback: try to parse from stats
        stats = self.get_api_data("/api/vegetation/stats")
        if stats and 'current_tick' in stats:
            return stats['current_tick']

        return 0

    def wait_for_simulator_ready(self):
        """Wait for simulator API to be available"""
        self.log("Waiting for simulator API to be ready...")
        max_wait = 30
        for i in range(max_wait):
            if self.get_api_data("/api/world_info"):
                self.log("Simulator API is ready!")
                return True
            time.sleep(1)
            if i % 5 == 0:
                self.log(f"  Still waiting... ({i}/{max_wait}s)")

        self.log("ERROR: Simulator API did not become ready in time")
        return False

    def start_simulator(self):
        """Start the life simulator process"""
        self.log("Starting Life Simulator...")

        # Clear old log
        if os.path.exists(SIM_LOG):
            os.remove(SIM_LOG)

        env = os.environ.copy()
        env['RUST_LOG'] = 'warn'

        with open(SIM_LOG, 'w') as log_out:
            self.sim_process = subprocess.Popen(
                ['./target/release/life-simulator'],
                stdout=log_out,
                stderr=subprocess.STDOUT,
                env=env
            )

        self.log(f"  Simulator PID: {self.sim_process.pid}")

        # Wait for API to be ready
        if not self.wait_for_simulator_ready():
            self.log("ERROR: Failed to start simulator")
            sys.exit(1)

        self.log("Simulator started successfully!")

    def take_sample(self):
        """Take a population sample"""
        populations = self.get_population_counts()
        if not populations:
            return False

        sample = {
            'tick': self.current_tick,
            'timestamp': time.time(),
            'elapsed_mins': (time.time() - self.start_time) / 60.0,
            'populations': populations,
            'total': sum(populations.values())
        }

        self.population_history.append(sample)

        # Log current state
        species_str = ", ".join([f"{k}: {v}" for k, v in sorted(populations.items())])
        self.log(
            f"Tick {self.current_tick:,}: Total={sample['total']} | {species_str}"
        )

        return True

    def monitor_loop(self):
        """Main monitoring loop"""
        self.log(f"Starting ecosystem balance monitoring...")
        self.log(f"  Target: {TARGET_TICKS:,} ticks")
        self.log(f"  Sample every: {SAMPLE_INTERVAL_TICKS} ticks")
        self.log(f"  Poll interval: {POLL_INTERVAL}s")
        self.log("")

        # Set starting tick on first poll
        if self.starting_tick is None:
            self.starting_tick = self.get_current_tick()
            self.log(f"  Starting from tick: {self.starting_tick:,}")
            self.last_sampled_tick = self.starting_tick

        try:
            while self.sim_process.poll() is None:
                # Get current tick (absolute)
                absolute_tick = self.get_current_tick()
                # Calculate relative tick from start
                self.current_tick = absolute_tick - self.starting_tick

                # Check if we need to sample
                relative_sample_tick = self.current_tick - (self.last_sampled_tick - self.starting_tick)
                if relative_sample_tick >= SAMPLE_INTERVAL_TICKS:
                    if self.take_sample():
                        self.last_sampled_tick = absolute_tick

                # Check if we've reached target
                if self.current_tick >= TARGET_TICKS:
                    self.log("")
                    self.log(f"Target of {TARGET_TICKS:,} relative ticks reached!")
                    self.log(f"  (Absolute tick: {absolute_tick:,})")
                    # Take final sample
                    self.take_sample()
                    break

                time.sleep(POLL_INTERVAL)

        except KeyboardInterrupt:
            self.log("")
            self.log("Monitoring interrupted by user")

        # Stop simulator
        self.log("Stopping simulator...")
        self.sim_process.terminate()
        try:
            self.sim_process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.sim_process.kill()
            self.sim_process.wait()

        self.log("Simulator stopped")

    def save_data(self):
        """Save population data to JSON"""
        with open(DATA_FILE, 'w') as f:
            json.dump({
                'target_ticks': TARGET_TICKS,
                'actual_ticks': self.current_tick,
                'samples': self.population_history,
                'runtime_minutes': (time.time() - self.start_time) / 60.0
            }, f, indent=2)
        self.log(f"Data saved to: {DATA_FILE}")

    def analyze_stability(self):
        """Analyze ecosystem stability"""
        if len(self.population_history) < 2:
            return {
                'stable': False,
                'message': 'Insufficient data for analysis',
                'issues': ['Insufficient data - no samples collected'],
                'extinctions': [],
                'fluctuations': {},
                'all_species': []
            }

        # Track all species that appeared
        all_species = set()
        for sample in self.population_history:
            all_species.update(sample['populations'].keys())

        # Check for extinctions
        extinctions = []
        final_pops = self.population_history[-1]['populations']
        for species in all_species:
            if species not in final_pops or final_pops[species] == 0:
                extinctions.append(species)

        # Check population fluctuations
        fluctuations = {}
        for species in all_species:
            pops = [s['populations'].get(species, 0) for s in self.population_history]
            if pops:
                min_pop = min(pops)
                max_pop = max(pops)
                avg_pop = sum(pops) / len(pops)
                fluctuations[species] = {
                    'min': min_pop,
                    'max': max_pop,
                    'avg': avg_pop,
                    'range': max_pop - min_pop,
                    'coefficient_of_variation': (max_pop - min_pop) / avg_pop if avg_pop > 0 else 0
                }

        # Assess stability
        stable = True
        issues = []

        if extinctions:
            stable = False
            issues.append(f"Species extinctions: {', '.join(extinctions)}")

        # Check for wild oscillations (CV > 1.0 is concerning)
        wild_oscillations = [s for s, f in fluctuations.items()
                            if f['coefficient_of_variation'] > 1.0 and s not in extinctions]
        if wild_oscillations:
            issues.append(f"Wild population oscillations: {', '.join(wild_oscillations)}")

        return {
            'stable': stable,
            'extinctions': extinctions,
            'fluctuations': fluctuations,
            'issues': issues,
            'all_species': sorted(list(all_species))
        }

    def calculate_predator_prey_ratios(self):
        """Calculate predator-prey ratios over time"""
        ratios = []

        for sample in self.population_history:
            pops = sample['populations']

            # Herbivores (prey)
            herbivores = pops.get('rabbit', 0) + pops.get('deer', 0)

            # Carnivores (predators)
            carnivores = pops.get('wolf', 0) + pops.get('fox', 0) + pops.get('bear', 0)

            ratio = herbivores / carnivores if carnivores > 0 else float('inf')

            ratios.append({
                'tick': sample['tick'],
                'herbivores': herbivores,
                'carnivores': carnivores,
                'ratio': ratio
            })

        return ratios

    def generate_report(self):
        """Generate comprehensive ecosystem balance report"""
        self.log("")
        self.log(f"Generating report: {REPORT_FILE}")

        stability = self.analyze_stability()
        ratios = self.calculate_predator_prey_ratios()

        elapsed = time.time() - self.start_time

        with open(REPORT_FILE, 'w') as f:
            f.write(f"""# Ecosystem Balance Test Results

**Test Date**: {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}

## Test Configuration

- **Target Ticks**: {TARGET_TICKS:,}
- **Actual Ticks Reached**: {self.current_tick:,}
- **Sample Interval**: Every {SAMPLE_INTERVAL_TICKS} ticks
- **Runtime**: {elapsed/60:.1f} minutes ({elapsed/3600:.2f} hours)
- **Average TPS**: {self.current_tick / elapsed:.2f}
- **Samples Collected**: {len(self.population_history)}

## Ecosystem Stability Assessment

""")

            if stability['stable']:
                f.write("### **STABLE** - Ecosystem is balanced and sustainable\n\n")
            else:
                f.write("### **UNSTABLE** - Issues detected\n\n")

            if stability['issues']:
                f.write("**Issues Detected**:\n")
                for issue in stability['issues']:
                    f.write(f"- {issue}\n")
                f.write("\n")
            else:
                f.write("**No major issues detected**\n\n")

            f.write(f"**Species Present**: {', '.join(stability['all_species'])}\n\n")

            if stability['extinctions']:
                f.write(f"**Species Extinctions**: {', '.join(stability['extinctions'])}\n\n")
            else:
                f.write("**No Species Extinctions** - All species survived\n\n")

            # Population fluctuations
            f.write("## Population Dynamics\n\n")
            f.write("### Population Statistics\n\n")
            f.write("| Species | Min | Max | Average | Range | CV |\n")
            f.write("|---------|-----|-----|---------|-------|----||\n")

            for species in sorted(stability['all_species']):
                if species in stability['fluctuations']:
                    f = stability['fluctuations'][species]
                    cv_status = "**HIGH**" if f['coefficient_of_variation'] > 1.0 else "OK"
                    f.write(f"| {species} | {f['min']:.0f} | {f['max']:.0f} | "
                           f"{f['avg']:.1f} | {f['range']:.0f} | "
                           f"{f['coefficient_of_variation']:.2f} {cv_status} |\n")

            # Predator-prey ratios
            f.write("\n### Predator-Prey Ratios\n\n")
            if ratios:
                avg_ratio = sum(r['ratio'] for r in ratios if r['ratio'] != float('inf')) / len([r for r in ratios if r['ratio'] != float('inf')])
                f.write(f"**Average Herbivore:Carnivore Ratio**: {avg_ratio:.2f}:1\n\n")
                f.write("Healthy ratio is typically 3:1 to 10:1 for sustainable ecosystems.\n\n")

                if avg_ratio < 2:
                    f.write("**WARNING**: Very low prey-to-predator ratio. Predators may struggle to find food.\n\n")
                elif avg_ratio > 15:
                    f.write("**WARNING**: Very high prey-to-predator ratio. Herbivore overpopulation possible.\n\n")
                else:
                    f.write("**OK**: Ratio is within reasonable range for ecosystem balance.\n\n")

            # Population timeline
            f.write("## Population Timeline\n\n")
            f.write("| Tick | Elapsed (min) | Rabbits | Deer | Wolves | Foxes | Bears | Total |\n")
            f.write("|------|---------------|---------|------|--------|-------|-------|-------|\n")

            for sample in self.population_history:
                pops = sample['populations']
                f.write(f"| {sample['tick']:,} | {sample['elapsed_mins']:.1f} | "
                       f"{pops.get('rabbit', 0)} | {pops.get('deer', 0)} | "
                       f"{pops.get('wolf', 0)} | {pops.get('fox', 0)} | "
                       f"{pops.get('bear', 0)} | {sample['total']} |\n")

            # Recommendations
            f.write("\n## Recommendations\n\n")

            if not stability['stable']:
                f.write("### Tuning Required\n\n")
                for issue in stability['issues']:
                    f.write(f"- Investigate: {issue}\n")

                if stability['extinctions']:
                    for species in stability['extinctions']:
                        f.write(f"- Increase {species} spawn rate or improve survival conditions\n")

                f.write("\n")
            else:
                f.write("### Ecosystem is Balanced\n\n")
                f.write("- Current spawn rates and parameters are sustainable\n")
                f.write("- All species maintained viable populations\n")
                f.write("- Predator-prey dynamics are stable\n\n")

            f.write("### Next Steps\n\n")
            f.write("- Continue monitoring in longer simulations (50k+ ticks)\n")
            f.write("- Test with different spawn configurations\n")
            f.write("- Validate behavior under stress conditions (resource scarcity)\n")
            f.write("- Monitor for memory leaks and performance degradation\n")

            # Raw data reference
            f.write(f"\n---\n\n**Detailed Data**: {DATA_FILE}\n")
            f.write(f"**Simulator Log**: {SIM_LOG}\n")

        self.log(f"Report saved to: {REPORT_FILE}")

    def run(self):
        """Run the complete ecosystem balance test"""
        self.log("=" * 60)
        self.log("ECOSYSTEM BALANCE TEST - 10,000 TICK SIMULATION")
        self.log("=" * 60)
        self.log("")

        try:
            self.start_simulator()
            self.monitor_loop()
            self.save_data()
            self.generate_report()

            self.log("")
            self.log("=" * 60)
            self.log("ECOSYSTEM BALANCE TEST COMPLETE")
            self.log("=" * 60)

        except Exception as e:
            self.log(f"ERROR: {e}")
            import traceback
            traceback.print_exc()
            sys.exit(1)

if __name__ == '__main__':
    test = EcosystemBalanceTest()
    test.run()
