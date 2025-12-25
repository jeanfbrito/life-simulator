//! Integration tests for sim-monitor

use sim_monitor::app::App;

#[tokio::test]
async fn test_full_update_cycle() {
    // Setup mock server
    let mut server = mockito::Server::new_async().await;

    // Mock entities endpoint
    let entities_mock = server
        .mock("GET", "/api/entities")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "entities": [
                {"id": 1, "species": "Rabbit", "x": 10.0, "y": 20.0, "health": 100.0},
                {"id": 2, "species": "Rabbit", "x": 15.0, "y": 25.0, "health": 95.0},
                {"id": 3, "species": "Fox", "x": 30.0, "y": 40.0, "health": 100.0}
            ]
        }"#,
        )
        .create_async()
        .await;

    // Mock health endpoint (called twice: once for connection check, once for data)
    let health_mock = server
        .mock("GET", "/api/debug/health")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "status": "ok",
            "alerts": {
                "tps_below_10": 0,
                "entities_stuck": 2,
                "population_crash": 0,
                "ai_loops": 1
            },
            "current_tps": 59.8,
            "total_alerts": 3,
            "is_healthy": true
        }"#,
        )
        .expect(2) // Connection check + data fetch
        .create_async()
        .await;

    // Mock alerts endpoint
    let alerts_mock = server
        .mock("GET", "/api/debug/alerts")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "alerts": [
                {
                    "tick": 1234,
                    "type": "EntitiesStuck",
                    "timestamp_ms": 1640000000000,
                    "message": "2 entities stuck"
                }
            ],
            "total": 1
        }"#,
        )
        .create_async()
        .await;

    // Mock TPS endpoint
    let tps_mock = server
        .mock("GET", "/api/debug/tps")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "current_tps": 59.8,
            "average_tps": 59.5,
            "status": "excellent"
        }"#,
        )
        .create_async()
        .await;

    // Create app and run update
    let mut app = App::new(server.url(), 1);
    let result = app.update().await;

    // Verify update succeeded
    assert!(result.is_ok(), "Update should succeed");

    // Verify app state
    assert!(app.is_connected, "App should be connected");
    assert_eq!(app.get_total_entities(), 3, "Should have 3 entities");
    assert_eq!(
        app.entity_counts.get("Rabbit"),
        Some(&2),
        "Should have 2 rabbits"
    );
    assert_eq!(app.entity_counts.get("Fox"), Some(&1), "Should have 1 fox");

    // Verify health status
    assert!(app.health_status.is_some(), "Should have health status");
    let health = app.health_status.as_ref().unwrap();
    assert_eq!(health.status, "ok");
    assert_eq!(health.current_tps, 59.8);

    // Verify TPS display
    assert_eq!(app.get_tps_display(), "59.8");

    // Verify connection status
    assert_eq!(app.get_connection_status(), "Connected");

    // Verify alerts
    assert_eq!(app.alerts.len(), 1, "Should have 1 alert");
    assert_eq!(app.alerts[0].alert_type, "EntitiesStuck");

    // Verify all mocks were called
    entities_mock.assert_async().await;
    health_mock.assert_async().await;
    alerts_mock.assert_async().await;
    tps_mock.assert_async().await;
}

#[tokio::test]
async fn test_delta_tracking_over_updates() {
    let mut server = mockito::Server::new_async().await;

    // First update - 3 rabbits
    let _first_mock = server
        .mock("GET", "/api/entities")
        .with_status(200)
        .with_body(
            r#"{
            "entities": [
                {"id": 1, "species": "Rabbit", "x": 10.0, "y": 20.0, "health": 100.0},
                {"id": 2, "species": "Rabbit", "x": 15.0, "y": 25.0, "health": 95.0},
                {"id": 3, "species": "Rabbit", "x": 20.0, "y": 30.0, "health": 90.0}
            ]
        }"#,
        )
        .create_async()
        .await;

    let _health_mock1 = server
        .mock("GET", "/api/debug/health")
        .with_status(200)
        .with_body(r#"{"status": "ok", "alerts": {}, "current_tps": 60.0}"#)
        .create_async()
        .await;

    let _alerts_mock1 = server
        .mock("GET", "/api/debug/alerts")
        .with_status(200)
        .with_body(r#"{"alerts": [], "total": 0}"#)
        .create_async()
        .await;

    let _tps_mock1 = server
        .mock("GET", "/api/debug/tps")
        .with_status(200)
        .with_body(r#"{"current_tps": 60.0, "average_tps": 60.0, "status": "excellent"}"#)
        .create_async()
        .await;

    let mut app = App::new(server.url(), 1);
    app.update().await.unwrap();

    assert_eq!(app.get_total_entities(), 3);
    assert_eq!(app.get_entity_delta("Rabbit"), 3); // New species, so delta is positive

    // Second update - 2 rabbits (one died)
    let _second_mock = server
        .mock("GET", "/api/entities")
        .with_status(200)
        .with_body(
            r#"{
            "entities": [
                {"id": 1, "species": "Rabbit", "x": 10.0, "y": 20.0, "health": 100.0},
                {"id": 2, "species": "Rabbit", "x": 15.0, "y": 25.0, "health": 95.0}
            ]
        }"#,
        )
        .create_async()
        .await;

    let _health_mock2 = server
        .mock("GET", "/api/debug/health")
        .with_status(200)
        .with_body(r#"{"status": "ok", "alerts": {}, "current_tps": 60.0}"#)
        .create_async()
        .await;

    let _alerts_mock2 = server
        .mock("GET", "/api/debug/alerts")
        .with_status(200)
        .with_body(r#"{"alerts": [], "total": 0}"#)
        .create_async()
        .await;

    let _tps_mock2 = server
        .mock("GET", "/api/debug/tps")
        .with_status(200)
        .with_body(r#"{"current_tps": 60.0, "average_tps": 60.0, "status": "excellent"}"#)
        .create_async()
        .await;

    app.update().await.unwrap();

    assert_eq!(app.get_total_entities(), 2);
    assert_eq!(app.get_entity_delta("Rabbit"), -1); // Lost one rabbit
}

#[tokio::test]
async fn test_disconnection_handling() {
    // Create app with invalid URL
    let mut app = App::new("http://invalid-host-12345:99999".to_string(), 1);

    // Update should succeed but mark as disconnected
    let result = app.update().await;
    assert!(result.is_ok(), "Update should not error on disconnection");
    assert!(!app.is_connected, "Should be disconnected");
    assert_eq!(app.get_connection_status(), "Disconnected");
}
