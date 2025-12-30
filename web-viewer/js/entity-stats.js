// Entity Stats Manager
// Handles fetching and displaying entity statistics in the right sidebar

import { ENTITY_CONFIG } from './config.js';

export class EntityStatsManager {
    constructor() {
        this.entities = [];
        this.updateInterval = 500; // Update every 500ms
        this.isRunning = false;
        // Stable ordering across updates
        this.entityOrder = new Map(); // id -> index
        this.nextOrderIndex = 0;
        // Track if click handler is already attached
        this.clickHandlerAttached = false;
    }

    // Sanitize HTML to prevent XSS attacks
    sanitizeHTML(str) {
        const div = document.createElement('div');
        div.textContent = str;
        return div.innerHTML;
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

        // Seed initial stable order, then append new entities in arrival order
        this.seedInitialOrder(this.entities);
        this.updateOrderIndices(this.entities);

        // Sort by stable order index
        const sorted = [...this.entities].sort((a, b) => {
            const ai = this.entityOrder.get(String(a.id)) ?? 0;
            const bi = this.entityOrder.get(String(b.id)) ?? 0;
            return ai - bi;
        });

        container.innerHTML = sorted.map(entity => this.renderEntityCard(entity)).join('');

        // Setup click handlers using event delegation (only once)
        if (!this.clickHandlerAttached) {
            container.addEventListener('click', (e) => this.handleEntityClick(e));
            this.clickHandlerAttached = true;
        }
    }

    /**
     * Handle click events on entity names using event delegation
     * @param {MouseEvent} e - Click event
     */
    handleEntityClick(e) {
        const nameElement = e.target.closest('.entity-name-clickable');
        if (!nameElement) return;

        const posX = parseInt(nameElement.dataset.posX, 10);
        const posY = parseInt(nameElement.dataset.posY, 10);

        if (isNaN(posX) || isNaN(posY)) {
            console.warn('Invalid entity position data');
            return;
        }

        // Access the global app instance to center the view
        if (window.lifeSimulatorApp && window.lifeSimulatorApp.controls) {
            window.lifeSimulatorApp.controls.centerOnEntity(posX, posY);
        } else {
            console.warn('LifeSimulatorApp not available');
        }
    }

    renderEntityCard(entity) {
        const emoji = this.getEntityEmoji(entity.entity_type);
        const actionLabel = entity.current_action ? this.renderCurrentAction(entity.current_action) : '';
        const safeName = this.sanitizeHTML(entity.name); // Sanitize entity name to prevent XSS
        const posX = entity.position?.x ?? 0;
        const posY = entity.position?.y ?? 0;

        return `
            <div class="entity-card" data-entity-id="${entity.id}">
                <div class="entity-header">
                    <div class="entity-name entity-name-clickable"
                         data-pos-x="${posX}"
                         data-pos-y="${posY}"
                         title="Click to center view on this entity">${safeName} ${this.renderSex(entity)}</div>
                    <div class="entity-type">${emoji}</div>
                </div>
                ${actionLabel}
                ${this.renderStats(entity)}
            </div>
        `;
    }

    renderCurrentAction(action) {
        const safeAction = this.sanitizeHTML(action); // Sanitize action to prevent XSS
        return `
            <div class="entity-action">
                <span class="action-label">Action:</span> <span class="action-value">${safeAction}</span>
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

        const reproductionStats = this.renderReproductionStats(entity);
        stats.push(...reproductionStats);

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
        const entityConfig = ENTITY_CONFIG[entityType];
        return entityConfig ? entityConfig.emoji : ENTITY_CONFIG.default.emoji;
    }

    // Maintain a stable initial ordering: Humans, Rabbits, then others by name
    seedInitialOrder(entities) {
        if (this.entityOrder.size > 0) return;
        const typeOrder = { 'Human': 0, 'Rabbit': 1, 'Deer': 2, 'Wolf': 3 };
        const initial = [...entities].sort((a, b) => {
            const at = typeOrder[a.entity_type] ?? 99;
            const bt = typeOrder[b.entity_type] ?? 99;
            if (at !== bt) return at - bt;
            return String(a.name || '').localeCompare(String(b.name || ''));
        });
        initial.forEach((e, idx) => {
            this.entityOrder.set(String(e.id), idx);
        });
        this.nextOrderIndex = initial.length;
    }

    // Add new entities at the end, keep existing indices
    updateOrderIndices(entities) {
        for (const e of entities) {
            const key = String(e.id);
            if (!this.entityOrder.has(key)) {
                this.entityOrder.set(key, this.nextOrderIndex++);
            }
        }
    }

    renderReproductionStats(entity) {
        const hasReproductionData = entity.sex !== undefined
            || entity.well_fed_streak !== undefined
            || entity.reproduction_cooldown_ticks !== undefined
            || entity.pregnancy_remaining_ticks !== undefined
            || entity.ticks_to_adult !== undefined;

        if (!hasReproductionData) return [];

        const TPS = 10; // ticks per second for display conversions

        const wf = entity.well_fed_streak ?? null;
        const wfReq = entity.well_fed_required_ticks ?? null;
        const wfValue = (wf !== null && wfReq !== null)
            ? Math.min(wf, wfReq)
            : wf;
        const wfPct = (wfValue !== null && wfReq !== null && wfReq > 0)
            ? Math.max(0, Math.min(100, (wfValue / wfReq) * 100))
            : null;

        const pregLeft = entity.pregnancy_remaining_ticks ?? null;
        const gestTotal = entity.gestation_total_ticks ?? null;
        const gestProgress = (pregLeft !== null && gestTotal && gestTotal > 0)
            ? Math.max(0, Math.min(100, ((gestTotal - pregLeft) / gestTotal) * 100))
            : null;

        const cd = entity.reproduction_cooldown_ticks ?? null;
        const eligible = entity.eligible_to_mate ?? null;
        const ticksToAdult = entity.ticks_to_adult ?? null;

        let status = null;
        if (pregLeft !== null) {
            status = `Pregnant (${Math.ceil(pregLeft / TPS)}s left)`;
        } else if (eligible !== null) {
            status = eligible ? 'Eligible to mate' : 'Not eligible';
        } else if (cd !== null && cd > 0) {
            status = `Cooldown: ${Math.ceil(cd / TPS)}s`;
        }

        const sections = [];

        if (status) {
            sections.push(`
                <div class="stat-bar-container stat-reproduction">
                    <div class="stat-bar-label">
                        <span>Reproduction</span>
                        <span style="font-weight: bold;">${status}</span>
                    </div>
                </div>
            `);
        }

        if (wf !== null && wfReq !== null && wfReq > 0) {
            const wfLabel = wf > wfReq ? `${wfReq}+/${wfReq}` : `${wf}/${wfReq}`;
            sections.push(`
                <div class="stat-bar-container stat-reproduction">
                    <div class="stat-bar-label">
                        <span>Well-fed streak</span>
                        <span>${wfLabel}</span>
                    </div>
                    <div class="stat-bar"><div class="stat-bar-fill" style="width:${wfPct.toFixed(0)}%"></div></div>
                </div>
            `);
        } else if (wf !== null && wfReq === null) {
            sections.push(`
                <div class="stat-bar-container stat-reproduction">
                    <div class="stat-bar-label">
                        <span>Well-fed streak</span>
                        <span>${wf} ticks</span>
                    </div>
                </div>
            `);
        }

        if (pregLeft !== null && gestTotal !== null && gestTotal > 0) {
            sections.push(`
                <div class="stat-bar-container stat-reproduction">
                    <div class="stat-bar-label">
                        <span>Gestation</span>
                        <span>${Math.ceil((gestTotal - pregLeft)/TPS)}s / ${Math.ceil(gestTotal/TPS)}s</span>
                    </div>
                    <div class="stat-bar"><div class="stat-bar-fill" style="width:${gestProgress.toFixed(0)}%"></div></div>
                </div>
            `);
        }

        if (cd !== null && cd > 0) {
            sections.push(`
                <div class="stat-bar-container stat-reproduction">
                    <div class="stat-bar-label">
                        <span>Cooldown</span>
                        <span>${Math.ceil(cd / TPS)}s</span>
                    </div>
                </div>
            `);
        }

        if (ticksToAdult !== null && ticksToAdult > 0) {
            sections.push(`
                <div class="stat-bar-container stat-reproduction">
                    <div class="stat-bar-label">
                        <span>Maturity</span>
                        <span>${Math.ceil(ticksToAdult / TPS)}s to adult</span>
                    </div>
                </div>
            `);
        }

        return sections;
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
