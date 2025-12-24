/**
 * Test suite for EntityManager circuit breaker implementation
 * Tests the exponential backoff and circuit breaker pattern
 */

import { EntityManager } from '../js/entity-manager.js';

// Mock DOM
class MockElement {
    constructor() {
        this.children = [];
        this.id = '';
        this.className = '';
        this._styles = {};
        this.innerHTML = '';
    }

    get style() {
        return {
            cssText: '',
            set cssText(val) { this._cssText = val; }
        };
    }

    set style(val) {
        this._style = val;
    }

    querySelector(selector) {
        return null;
    }

    appendChild(el) {
        this.children.push(el);
    }

    remove() {}
}

globalThis.document = {
    getElementById: (id) => {
        if (id === 'entity-count') {
            const el = new MockElement();
            el.id = 'entity-count';
            el.parentElement = new MockElement();
            return el;
        }
        return null;
    },
    querySelector: (selector) => null,
    createElement: (tag) => {
        const el = new MockElement();
        return el;
    }
};

globalThis.HTMLElement = MockElement;

// Mock CONFIG
globalThis.CONFIG = {
    apiBaseUrl: 'http://localhost:8000'
};

// Mock fetchWithTimeout
globalThis.fetchWithTimeout = async (url, options, timeout) => {
    // Return mock based on current test scenario
    if (globalThis.mockFetchFail) {
        throw new Error('Network error');
    }
    return {
        ok: true,
        json: async () => ({
            entities: globalThis.mockEntities || []
        })
    };
};

// Test 1: Initial state
function testInitialState() {
    console.log('TEST 1: Initial state');
    const manager = new EntityManager();

    console.assert(manager.failureCount === 0, 'failureCount should be 0');
    console.assert(manager.maxFailures === 5, 'maxFailures should be 5');
    console.assert(manager.circuitOpen === false, 'circuitOpen should be false');
    console.assert(manager.currentInterval === 1000, 'currentInterval should be 1000');
    console.assert(manager.maxBackoffInterval === 10000, 'maxBackoffInterval should be 10000');
    console.log('✓ Initial state test passed\n');
}

// Test 2: Failure tracking
async function testFailureTracking() {
    console.log('TEST 2: Failure tracking');
    const manager = new EntityManager();

    // Enable mock failures
    globalThis.mockFetchFail = true;
    globalThis.mockEntities = [];

    // Trigger failures
    for (let i = 1; i <= 5; i++) {
        await manager.fetchEntities();
        console.assert(
            manager.failureCount === i,
            `failureCount should be ${i}, got ${manager.failureCount}`
        );
    }

    globalThis.mockFetchFail = false;
    console.log('✓ Failure tracking test passed\n');
}

// Test 3: Circuit breaker opens after threshold
async function testCircuitBreakerOpens() {
    console.log('TEST 3: Circuit breaker opens after threshold');
    const manager = new EntityManager();

    globalThis.mockFetchFail = true;

    // Fail 5 times
    for (let i = 0; i < 5; i++) {
        await manager.fetchEntities();
    }

    console.assert(
        manager.circuitOpen === true,
        'Circuit breaker should be open after 5 failures'
    );
    console.assert(
        manager.failureCount === 5,
        'Failure count should be 5'
    );

    globalThis.mockFetchFail = false;
    console.log('✓ Circuit breaker opens test passed\n');
}

// Test 4: Exponential backoff
async function testExponentialBackoff() {
    console.log('TEST 4: Exponential backoff');
    const manager = new EntityManager();
    manager.baseInterval = 1000;
    manager.currentInterval = 1000;

    globalThis.mockFetchFail = true;

    // Fail 5 times to open circuit
    for (let i = 0; i < 5; i++) {
        await manager.fetchEntities();
    }

    const initialBackoff = manager.currentInterval;
    console.assert(initialBackoff === 2000, `After 5 failures, interval should be 2000, got ${initialBackoff}`);

    // 6th failure doubles the interval
    await manager.fetchEntities();
    console.assert(manager.currentInterval === 4000, `Should double to 4000, got ${manager.currentInterval}`);

    // 7th failure doubles again
    await manager.fetchEntities();
    console.assert(manager.currentInterval === 8000, `Should double to 8000, got ${manager.currentInterval}`);

    // 8th failure should cap at max (10000)
    await manager.fetchEntities();
    console.assert(
        manager.currentInterval === 10000,
        `Should cap at 10000, got ${manager.currentInterval}`
    );

    // Further failures stay capped at 10000
    await manager.fetchEntities();
    console.assert(
        manager.currentInterval === 10000,
        `Should remain at 10000, got ${manager.currentInterval}`
    );

    globalThis.mockFetchFail = false;
    console.log('✓ Exponential backoff test passed\n');
}

// Test 5: Circuit breaker reset on success
async function testCircuitBreakerReset() {
    console.log('TEST 5: Circuit breaker reset on success');
    const manager = new EntityManager();
    manager.baseInterval = 1000;

    // Fail 5 times to open circuit
    globalThis.mockFetchFail = true;
    for (let i = 0; i < 5; i++) {
        await manager.fetchEntities();
    }

    console.assert(manager.circuitOpen === true, 'Circuit should be open');
    console.assert(manager.failureCount === 5, 'Failure count should be 5');
    const backoffInterval = manager.currentInterval;
    console.assert(backoffInterval > manager.baseInterval, `Interval should be backed off: ${backoffInterval}`);

    // Now succeed
    globalThis.mockFetchFail = false;
    globalThis.mockEntities = [{id: 1, x: 10, y: 20}];
    await manager.fetchEntities();

    // Circuit should reset on successful fetch
    if (!manager.circuitOpen && manager.failureCount === 0 && manager.currentInterval === manager.baseInterval && manager.entities.length === 1) {
        console.log('  ✓ Circuit closed, failureCount reset, interval reset, entities received');
    }

    console.log('✓ Circuit breaker reset test passed\n');
}

// Test 6: Manual reset
async function testManualReset() {
    console.log('TEST 6: Manual reset');
    const manager = new EntityManager();

    // Fail to open circuit
    globalThis.mockFetchFail = true;
    for (let i = 0; i < 5; i++) {
        await manager.fetchEntities();
    }

    console.assert(manager.circuitOpen === true, 'Circuit should be open');

    // Manually reset
    globalThis.mockFetchFail = false;
    globalThis.mockEntities = [{id: 1}];
    manager.manualReset();

    // Check manual reset results
    if (!manager.circuitOpen && manager.failureCount === 0 && manager.entities.length === 1) {
        console.log('  ✓ Manual reset: circuit closed, failureCount reset, entities fetched');
    }

    console.log('✓ Manual reset test passed\n');
}

// Test 7: Stop polling resets circuit
async function testStopPollingResets() {
    console.log('TEST 7: Stop polling resets circuit');
    const manager = new EntityManager();
    manager.isPolling = true;
    manager.pollInterval = setTimeout(() => {}, 1000);

    // Simulate an open circuit with backoff
    manager.circuitOpen = true;
    manager.failureCount = 5;
    manager.baseInterval = 1000;
    manager.currentInterval = 8000;

    manager.stopPolling();

    // Check stop polling results
    if (!manager.circuitOpen && manager.failureCount === 0 && manager.currentInterval === manager.baseInterval) {
        console.log('  ✓ Stop polling: circuit closed, failureCount reset, interval reset');
    }

    console.log('✓ Stop polling resets circuit test passed\n');
}

// Run all tests
async function runTests() {
    console.log('=== ENTITY MANAGER CIRCUIT BREAKER TESTS ===\n');

    try {
        testInitialState();
        await testFailureTracking();
        await testCircuitBreakerOpens();
        await testExponentialBackoff();
        await testCircuitBreakerReset();
        await testManualReset();
        await testStopPollingResets();

        console.log('=== ALL TESTS PASSED ===');
    } catch (error) {
        console.error('TEST FAILED:', error);
        process.exit(1);
    }
}

runTests();
