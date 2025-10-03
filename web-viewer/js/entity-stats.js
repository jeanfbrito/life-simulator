// Entity Stats Manager
// Handles fetching and displaying entity statistics in the right sidebar

export class EntityStatsManager {
    constructor() {
        this.entities = [];
        this.updateInterval = 500; // Update every 500ms
        this.isRunning = false;
    }

    start() {
        if (this.isRunning) return;
        this.isRunning = true;
        this.update();
    }

    stop() {
        this.isRunning = false;
    }

    async update() {
        if (!this.isRunning) return;

        try {
            const response = await fetch('/api/entities');
            const data = await response.json();
            this.entities = data.entities || [];
            this.render();
        } catch (error) {
            console.warn('Failed to fetch entities:', error);
        }

        // Schedule next update
        setTimeout(() => this.update(), this.updateInterval);
    }

    render() {
        const container = document.getElementById('entity-list');
        if (!container) return;

        if (this.entities.length === 0) {
            container.innerHTML = `
                <div style="text-align: center; opacity: 0.5; padding: 2rem;">
                    No entities found
                </div>
            `;
            return;
        }

        // Sort entities by type (Humans first, then Rabbits)
        const sorted = [...this.entities].sort((a, b) => {
            if (a.entity_type === b.entity_type) {
                return a.name.localeCompare(b.name);
            }
            return a.entity_type === 'Human' ? -1 : 1;
        });

        container.innerHTML = sorted.map(entity => this.renderEntityCard(entity)).join('');
    }

    renderEntityCard(entity) {
        const emoji = this.getEntityEmoji(entity.entity_type);
        const actionLabel = entity.current_action ? this.renderCurrentAction(entity.current_action) : '';
        
        return `
            <div class="entity-card">
                <div class="entity-header">
                    <div class="entity-name">${entity.name} ${this.renderSex(entity)}</div>
                    <div class="entity-type">${emoji}</div>
                </div>
                ${actionLabel}
                ${this.renderStats(entity)}
                ${this.renderReproduction(entity)}
            </div>
        `;
    }

    renderCurrentAction(action) {
        return `
            <div class="entity-action">
                <span class="action-label">Action:</span> <span class="action-value">${action}</span>
            </div>
        `;
    }

    renderSex(entity) {
        const sex = entity.sex;
        if (sex === 'male') return '<span title="Male">♂</span>';
        if (sex === 'female') return '<span title="Female">♀</span>';
        return '';
    }

    renderStats(entity) {
        const stats = [];

        // Hunger (higher = more hungry, red/orange)
        if (entity.hunger !== undefined) {
            stats.push(this.renderStatBar('Hunger', entity.hunger, 'hunger', true));
        }

        // Thirst (higher = more thirsty, blue)
        if (entity.thirst !== undefined) {
            stats.push(this.renderStatBar('Thirst', entity.thirst, 'thirst', true));
        }

        // Energy (higher = more energy, green)
        if (entity.energy !== undefined) {
            stats.push(this.renderStatBar('Energy', entity.energy, 'energy', false));
        }

        // Health (higher = healthier, red to green gradient)
        if (entity.health !== undefined) {
            stats.push(this.renderStatBar('Health', entity.health, 'health', false));
        }

        return stats.length > 0 ? stats.join('') : '<div style="font-size: 0.75rem; opacity: 0.5;">No stats available</div>';
    }

    renderStatBar(label, value, className, isNeed) {
        // Clamp value between 0 and 100
        const percentage = Math.max(0, Math.min(100, value));
        
        // For needs (hunger/thirst), we want to show urgency
        // For resources (energy/health), we show fullness
        const displayValue = percentage.toFixed(0);
        
        // Determine if stat is critical (low for resources, high for needs)
        const isCritical = isNeed 
            ? percentage > 70  // Need is critical when high
            : percentage < 30; // Resource is critical when low
            
        const isWarning = isNeed
            ? percentage > 40 && percentage <= 70
            : percentage >= 30 && percentage < 60;
        
        const statusClass = isCritical ? 'critical' : (isWarning ? 'warning' : '');
        
        return `
            <div class="stat-bar-container stat-${className}">
                <div class="stat-bar-label">
                    <span>${label}</span>
                    <span style="font-weight: bold; ${isCritical ? 'color: #ef4444;' : ''}">${displayValue}%</span>
                </div>
                <div class="stat-bar">
                    <div class="stat-bar-fill" style="width: ${percentage}%;"></div>
                </div>
            </div>
        `;
    }

    getEntityEmoji(entityType) {
        const emojis = {
            'Human': '🧍‍♂️',
            'Rabbit': '🐇',
            'Deer': '🦌',
            'Wolf': '🐺'
        };
        return emojis[entityType] || '❓';
    }

    // Collapsible reproduction diagnostics (for rabbits)
    renderReproduction(entity) {
        if (entity.entity_type !== 'Rabbit') return '';
        const TPS = 10; // display conversion

        const wf = entity.well_fed_streak ?? null;
        const wfReq = entity.well_fed_required_ticks ?? null;
        const wfPct = (wf !== null && wfReq) ? Math.max(0, Math.min(100, (wf / wfReq) * 100)) : null;

        const pregLeft = entity.pregnancy_remaining_ticks ?? null;
        const gestTotal = entity.gestation_total_ticks ?? null;
        const pregPct = (pregLeft !== null && gestTotal) ? Math.max(0, Math.min(100, ((gestTotal - pregLeft) / gestTotal) * 100)) : null;

        const cd = entity.reproduction_cooldown_ticks ?? null;
        const eligible = entity.eligible_to_mate ?? null;
        const ticksToAdult = entity.ticks_to_adult ?? null;

        const status = pregLeft !== null ? `Pregnant (${Math.ceil(pregLeft / TPS)}s left)`
                      : (eligible ? 'Eligible to mate' : (cd ? `Cooldown: ${Math.ceil(cd / TPS)}s` : 'Not eligible'));

        const wfSection = (wf !== null && wfReq !== null) ? `
            <div class="stat-bar-container">
                <div class="stat-bar-label">
                    <span>Well-fed streak</span>
                    <span>${wf}/${wfReq}</span>
                </div>
                <div class="stat-bar"><div class="stat-bar-fill" style="width:${wfPct.toFixed(0)}%"></div></div>
            </div>` : '';

        const pregSection = (pregLeft !== null && gestTotal !== null) ? `
            <div class="stat-bar-container">
                <div class="stat-bar-label">
                    <span>Gestation</span>
                    <span>${Math.ceil((gestTotal - pregLeft)/TPS)}s / ${Math.ceil(gestTotal/TPS)}s</span>
                </div>
                <div class="stat-bar"><div class="stat-bar-fill" style="width:${pregPct.toFixed(0)}%"></div></div>
            </div>` : '';

        const cdSection = (cd !== null && cd > 0) ? `
            <div class="stat-bar-container">
                <div class="stat-bar-label">
                    <span>Cooldown</span>
                    <span>${Math.ceil(cd / TPS)}s</span>
                </div>
                <div class="stat-bar"><div class="stat-bar-fill" style="width:${Math.max(0, Math.min(100, (1 - (cd / (wfReq || cd))) * 100)).toFixed(0)}%"></div></div>
            </div>` : '';

        const maturitySection = (ticksToAdult !== null && ticksToAdult > 0) ? `
            <div class="stat-bar-container">
                <div class="stat-bar-label">
                    <span>Maturity</span>
                    <span>${Math.ceil(ticksToAdult / TPS)}s</span>
                </div>
            </div>` : '';

        return `
            <details class="entity-repro" style="margin-top:0.5rem;">
              <summary style="cursor:pointer; opacity:0.9;">Reproduction · <span style="font-weight:600;">${status}</span></summary>
              <div style="margin-top:0.35rem;">
                ${wfSection}
                ${pregSection}
                ${cdSection}
                ${maturitySection}
              </div>
            </details>
        `;
    }
}

// Initialize when DOM is ready
let statsManager;

export function initEntityStats() {
    statsManager = new EntityStatsManager();
    statsManager.start();
    console.log('✅ Entity stats manager initialized');
    return statsManager;
}

export function getStatsManager() {
    return statsManager;
}
