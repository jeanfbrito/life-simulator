# Circuit Breaker Pattern - Entity Polling

## Overview

The EntityManager now implements the **Circuit Breaker Pattern** with exponential backoff to prevent flooding the API server with requests when it becomes unavailable.

## Problem Solved

**Before**: Entity polling would continue indefinitely, even when the API server was down, creating an exponential flood of failed requests that could overwhelm the server during recovery.

**After**: The circuit breaker monitors failures and automatically backs off, reducing server load during outages.

## How It Works

### States

The circuit breaker has three states:

1. **CLOSED** (Normal)
   - Polling continues at regular intervals
   - All failures count toward threshold
   - Failures are logged but don't affect polling

2. **OPEN** (Circuit Broken)
   - Triggered after 5 consecutive failures
   - Exponential backoff begins
   - Error UI appears to user
   - Retry interval doubles on each failure (1s → 2s → 4s → 8s → 10s max)

3. **RESET**
   - Triggered by a successful request
   - Failure count resets to 0
   - Interval returns to base value
   - Error UI removed
   - Normal polling resumes

### Configuration

Key parameters in EntityManager constructor:

```javascript
this.failureCount = 0;           // Tracks consecutive failures
this.maxFailures = 5;            // Threshold to open circuit
this.baseInterval = 1000;        // Normal polling interval (ms)
this.currentInterval = 1000;     // Current polling interval (may be backed off)
this.maxBackoffInterval = 10000; // Maximum backoff interval (10 seconds)
this.circuitOpen = false;        // Circuit state flag
```

## Implementation Details

### Failure Tracking

Each failed fetch increments `failureCount`:

```javascript
handleFetchFailure(errorMessage) {
    this.failureCount++;
    console.warn(`Fetch failed (${this.failureCount}/${this.maxFailures}): ${errorMessage}`);

    if (this.failureCount >= this.maxFailures && !this.circuitOpen) {
        this.circuitOpen = true;
        this.showErrorUI();
    }
}
```

### Exponential Backoff

When circuit is open, each failure doubles the retry interval:

```javascript
if (this.circuitOpen) {
    this.currentInterval = Math.min(
        this.currentInterval * 2,
        this.maxBackoffInterval
    );
    console.log(`Exponential backoff - next retry in ${this.currentInterval}ms`);
}
```

**Backoff Timeline**:
- Failures 1-5: Normal interval (1000ms)
- Failure 6: 2000ms
- Failure 7: 4000ms
- Failure 8: 8000ms
- Failure 9+: 10000ms (capped)

### Reset on Success

Any successful request resets the circuit:

```javascript
resetCircuitBreaker() {
    if (this.circuitOpen || this.failureCount > 0) {
        console.log('Circuit breaker CLOSED - connection restored');
        this.circuitOpen = false;
        this.failureCount = 0;
        this.currentInterval = this.baseInterval;

        // Remove error UI
        const errorIndicator = document.querySelector('.connection-error');
        if (errorIndicator) {
            errorIndicator.remove();
        }
    }
}
```

## User Feedback

### Error UI

When the circuit breaker opens (5 failures), users see:

```
⚠️ Connection Issues
API unreachable. Retrying with exponential backoff...
```

This appears below the entity count display with red styling.

### Console Logging

All state changes are logged with the `🎯 ENTITY_MANAGER:` prefix:

```
🎯 ENTITY_MANAGER: Fetch failed (1/5): Network error
🎯 ENTITY_MANAGER: Fetch failed (5/5): Network error
🎯 ENTITY_MANAGER: Circuit breaker OPENED after 5 failures
🎯 ENTITY_MANAGER: Exponential backoff - next retry in 2000ms
🎯 ENTITY_MANAGER: Circuit breaker CLOSED - connection restored
```

## Recovery Mechanisms

### Automatic Recovery

The circuit breaker automatically closes when the API becomes available again:

1. Next scheduled poll succeeds
2. `resetCircuitBreaker()` is called
3. Interval returns to normal
4. Error UI is removed
5. Polling resumes at regular pace

### Manual Recovery

Call `manualReset()` to force immediate recovery check:

```javascript
// From browser console or external control
window.lifeSimulatorApp.entityManager.manualReset();
```

This immediately attempts a fetch and resets the circuit if successful.

## Benefits

1. **Prevents Server Flooding**: Stops hammering a down server with hundreds of requests per second

2. **Faster Recovery**: Reduces network traffic during outages, allowing servers to recover faster

3. **User Awareness**: Clear visual feedback when API is unavailable

4. **Graceful Degradation**: Last known entity state continues to display to users

5. **Automatic Restoration**: No manual intervention needed - polls resume when API is back

## Testing

Run the test suite:

```bash
# In web-viewer directory
node tests/entity-manager.test.js
```

Tests verify:
- Initial state configuration
- Failure count tracking
- Circuit breaker opening at threshold
- Exponential backoff calculation
- Reset on successful request
- Manual reset functionality
- State reset on stop polling

## Integration with Application

EntityManager is initialized in `app.js`:

```javascript
this.entityManager = new EntityManager();

// Later, after initialization:
this.entityManager.startPolling(200); // Poll every 0.2 seconds
```

The polling can be stopped with:

```javascript
this.entityManager.stopPolling();
```

## Example Scenarios

### Scenario 1: API Temporarily Down

1. User visits viewer, polling starts normally (1s interval)
2. API becomes unavailable
3. Failures 1-4: Logged, polling continues
4. Failure 5: Circuit opens, error UI appears, backoff begins (2s)
5. Failure 6: Backoff increases to 4s
6. Meanwhile, API admin restarts the service
7. Next poll succeeds
8. Circuit closes, error UI disappears
9. Polling resumes at 1s interval

### Scenario 2: Network Connectivity Issue

1. User loses internet connection
2. Failures accumulate, circuit opens (5 failures)
3. Backoff increases to 10s between retries
4. User regains internet
5. Next scheduled poll succeeds
6. Circuit resets, polling resumes normally

## Performance Impact

- **Memory**: Minimal overhead (6 additional properties in EntityManager)
- **CPU**: No additional processing, same fetch cycle
- **Network**: Dramatic reduction during outages (from 5 req/sec to 0.1 req/sec when circuit open)
- **UI**: Single error message div added/removed during circuit state changes

## Future Enhancements

Potential improvements:

1. **Half-open state**: Test connectivity before fully closing circuit
2. **Configurable thresholds**: Allow customization per environment
3. **Circuit breaker dashboard**: Display circuit state in UI
4. **Metrics collection**: Track outage duration and recovery time
5. **Fallback data source**: Use cached data from last 24 hours if API down
6. **Alert notifications**: Notify admins when circuit opens
