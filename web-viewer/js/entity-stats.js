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
        
        return `
            <div class="entity-card">
                <div class="entity-header">
                    <div class="entity-name">${entity.name}</div>
                    <div class="entity-type">${emoji}</div>
                </div>
                ${this.renderStats(entity)}
            </div>
        `;
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
