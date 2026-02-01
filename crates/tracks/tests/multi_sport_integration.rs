//! Integration tests for multi-sport activity support.
//!
//! These tests verify end-to-end functionality including:
//! - Single-sport uploads with the new activity_type_id system
//! - Multi-sport uploads with type_boundaries and segment_types
//! - Custom activity type creation
//! - Segment matching respecting activity type boundaries
//!
//! To run these tests, you need:
//! 1. A PostgreSQL database with PostGIS extension and migrations applied
//! 2. DATABASE_URL environment variable set
//!
//! Run with: `DATABASE_URL=postgres://... cargo nextest run -p tracks multi_sport`
//!
//! Note: These tests create and clean up their own data using unique IDs,
//! so they can safely run against a development database.

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use time::{Duration, OffsetDateTime};
use tracks::database::Database;
use tracks::models::{Activity, Visibility, builtin_types};
use uuid::Uuid;

/// Get database pool, skipping tests if DATABASE_URL is not set.
async fn get_test_pool() -> Option<PgPool> {
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping test: DATABASE_URL not set");
            return None;
        }
    };

    match PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
    {
        Ok(pool) => Some(pool),
        Err(e) => {
            eprintln!("Skipping test: Failed to connect to database: {e}");
            None
        }
    }
}

/// Helper to create a test user in the database.
async fn create_test_user(pool: &PgPool, test_id: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO users (id, name, email, password_hash, auth_provider, created_at)
        VALUES ($1, $2, $3, 'hash', 'email', NOW())
        "#,
    )
    .bind(user_id)
    .bind(format!("Test User {test_id}"))
    .bind(format!("test-{test_id}-{}@example.com", user_id))
    .execute(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

/// Cleanup helper to remove test data.
async fn cleanup_test_data(pool: &PgPool, user_id: Uuid) {
    // Delete in order due to foreign key constraints
    let _ = sqlx::query("DELETE FROM segment_efforts WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM tracks WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM scores WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM activities WHERE user_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM segments WHERE creator_id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM activity_types WHERE created_by = $1")
        .bind(user_id)
        .execute(pool)
        .await;
    let _ = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await;
}

/// Helper to create a simple single-sport activity.
async fn create_single_sport_activity(db: &Database, user_id: Uuid) -> Activity {
    let now = OffsetDateTime::now_utc();
    let activity = Activity {
        id: Uuid::new_v4(),
        user_id,
        activity_type_id: builtin_types::RUN,
        name: "Morning Run".to_string(),
        object_store_path: format!("test/{}.gpx", Uuid::new_v4()),
        started_at: now,
        submitted_at: now,
        visibility: Visibility::Public.as_str().to_string(),
        type_boundaries: None,
        segment_types: None,
    };
    db.save_activity(&activity)
        .await
        .expect("Failed to save activity");
    activity
}

/// Helper to create a multi-sport activity.
async fn create_multi_sport_activity(
    db: &Database,
    user_id: Uuid,
    boundaries: Vec<OffsetDateTime>,
    segment_types: Vec<Uuid>,
) -> Activity {
    let now = OffsetDateTime::now_utc();
    let activity = Activity {
        id: Uuid::new_v4(),
        user_id,
        activity_type_id: segment_types[0], // Primary type is first segment
        name: "Multi-Sport Adventure".to_string(),
        object_store_path: format!("test/{}.gpx", Uuid::new_v4()),
        started_at: now,
        submitted_at: now,
        visibility: Visibility::Public.as_str().to_string(),
        type_boundaries: Some(boundaries),
        segment_types: Some(segment_types),
    };
    db.save_activity(&activity)
        .await
        .expect("Failed to save activity");
    activity
}

/// Helper to create a test segment.
#[allow(dead_code)]
async fn create_test_segment(
    pool: &PgPool,
    creator_id: Uuid,
    activity_type_id: Uuid,
    start_lat: f64,
    start_lon: f64,
    end_lat: f64,
    end_lon: f64,
) -> Uuid {
    let segment_id = Uuid::new_v4();

    // Create simple WKT geometry
    let geo_wkt = format!(
        "LINESTRING Z({} {} 1650, {} {} 1660)",
        start_lon, start_lat, end_lon, end_lat
    );
    let start_wkt = format!("POINT({} {})", start_lon, start_lat);
    let end_wkt = format!("POINT({} {})", end_lon, end_lat);

    sqlx::query(
        r#"
        INSERT INTO segments (
            id, creator_id, name, activity_type_id,
            geo, start_point, end_point,
            distance_meters, visibility, created_at
        )
        VALUES (
            $1, $2, 'Test Segment', $3,
            ST_GeogFromText($4), ST_GeogFromText($5), ST_GeogFromText($6),
            500.0, 'public', NOW()
        )
        "#,
    )
    .bind(segment_id)
    .bind(creator_id)
    .bind(activity_type_id)
    .bind(&geo_wkt)
    .bind(&start_wkt)
    .bind(&end_wkt)
    .execute(pool)
    .await
    .expect("Failed to create test segment");

    segment_id
}

/// Helper to create a track geometry for an activity.
#[allow(dead_code)]
async fn create_track_geometry(
    pool: &PgPool,
    user_id: Uuid,
    activity_id: Uuid,
    points: &[(f64, f64, f64, i64)], // (lat, lon, elevation, epoch_seconds)
) {
    let coords: Vec<String> = points
        .iter()
        .map(|(lat, lon, ele, epoch)| format!("{lon} {lat} {ele} {epoch}"))
        .collect();
    let wkt = format!("LINESTRING ZM({})", coords.join(", "));

    sqlx::query(
        r#"
        INSERT INTO tracks (user_id, activity_id, geo, created_at)
        VALUES ($1, $2, ST_GeogFromText($3), NOW())
        "#,
    )
    .bind(user_id)
    .bind(activity_id)
    .bind(&wkt)
    .execute(pool)
    .await
    .expect("Failed to create track geometry");
}

// ============================================================================
// Test 9.1: Single-sport upload with new type system
// ============================================================================

#[tokio::test]
async fn test_single_sport_activity_creation() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "single_sport").await;

    // Create a single-sport activity (no type_boundaries, no segment_types)
    let activity = create_single_sport_activity(&db, user_id).await;

    // Verify it was stored correctly
    let retrieved = db
        .get_activity(activity.id)
        .await
        .expect("Failed to get activity")
        .expect("Activity not found");

    assert_eq!(retrieved.id, activity.id);
    assert_eq!(retrieved.activity_type_id, builtin_types::RUN);
    assert!(retrieved.type_boundaries.is_none());
    assert!(retrieved.segment_types.is_none());

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_single_sport_activity_with_builtin_types() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "builtin_types").await;

    // Test with different built-in types
    let types_to_test = [
        (builtin_types::WALK, "walk"),
        (builtin_types::RUN, "run"),
        (builtin_types::HIKE, "hike"),
        (builtin_types::MTB, "mtb"),
        (builtin_types::ROAD, "road"),
        (builtin_types::GRAVEL, "gravel"),
    ];

    for (type_id, type_name) in types_to_test {
        let now = OffsetDateTime::now_utc();
        let activity = Activity {
            id: Uuid::new_v4(),
            user_id,
            activity_type_id: type_id,
            name: format!("{type_name} activity"),
            object_store_path: format!("test/{}.gpx", Uuid::new_v4()),
            started_at: now,
            submitted_at: now,
            visibility: Visibility::Public.as_str().to_string(),
            type_boundaries: None,
            segment_types: None,
        };

        db.save_activity(&activity)
            .await
            .expect("Failed to save activity");

        let retrieved = db
            .get_activity(activity.id)
            .await
            .expect("Failed to get activity")
            .expect("Activity not found");

        assert_eq!(
            retrieved.activity_type_id, type_id,
            "Type mismatch for {type_name}"
        );
    }

    cleanup_test_data(&pool, user_id).await;
}

// ============================================================================
// Test 9.2: Multi-sport activity upload
// ============================================================================

#[tokio::test]
async fn test_multi_sport_activity_creation() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "multi_sport").await;

    let start = OffsetDateTime::now_utc();
    let boundaries = vec![
        start,
        start + Duration::minutes(30),
        start + Duration::minutes(60),
    ];
    let segment_types = vec![builtin_types::RUN, builtin_types::MTB];

    let activity =
        create_multi_sport_activity(&db, user_id, boundaries.clone(), segment_types.clone()).await;

    // Verify it was stored correctly
    let retrieved = db
        .get_activity(activity.id)
        .await
        .expect("Failed to get activity")
        .expect("Activity not found");

    assert_eq!(retrieved.id, activity.id);
    assert_eq!(retrieved.activity_type_id, builtin_types::RUN); // Primary type

    // Verify boundaries and types are stored
    let stored_boundaries = retrieved.type_boundaries.expect("Should have boundaries");
    let stored_types = retrieved.segment_types.expect("Should have segment types");

    assert_eq!(stored_boundaries.len(), 3);
    assert_eq!(stored_types.len(), 2);
    assert_eq!(stored_types[0], builtin_types::RUN);
    assert_eq!(stored_types[1], builtin_types::MTB);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_multi_sport_activity_with_three_segments() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "three_segments").await;

    let start = OffsetDateTime::now_utc();
    // RUN -> MTB -> RUN pattern (common for trail run + bike shuttle + trail run)
    let boundaries = vec![
        start,
        start + Duration::minutes(30),
        start + Duration::minutes(60),
        start + Duration::minutes(90),
    ];
    let segment_types = vec![builtin_types::RUN, builtin_types::MTB, builtin_types::RUN];

    let activity =
        create_multi_sport_activity(&db, user_id, boundaries.clone(), segment_types.clone()).await;

    let retrieved = db
        .get_activity(activity.id)
        .await
        .expect("Failed to get activity")
        .expect("Activity not found");

    let stored_boundaries = retrieved.type_boundaries.expect("Should have boundaries");
    let stored_types = retrieved.segment_types.expect("Should have segment types");

    assert_eq!(
        stored_boundaries.len(),
        4,
        "Should have 4 boundaries for 3 segments"
    );
    assert_eq!(stored_types.len(), 3, "Should have 3 segment types");
    assert_eq!(stored_types[0], builtin_types::RUN);
    assert_eq!(stored_types[1], builtin_types::MTB);
    assert_eq!(stored_types[2], builtin_types::RUN);

    cleanup_test_data(&pool, user_id).await;
}

// ============================================================================
// Test 9.3: Custom activity type creation
// ============================================================================

#[tokio::test]
async fn test_create_custom_activity_type() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "custom_type").await;

    // Use unique name to avoid conflicts
    let type_name = format!("custom_type_{}", Uuid::new_v4().simple());

    let custom_type = db
        .create_activity_type(&type_name, user_id)
        .await
        .expect("Failed to create custom type");

    assert_eq!(custom_type.name, type_name);
    assert!(!custom_type.is_builtin);
    assert_eq!(custom_type.created_by, Some(user_id));

    // Verify we can use it in an activity
    let now = OffsetDateTime::now_utc();
    let activity = Activity {
        id: Uuid::new_v4(),
        user_id,
        activity_type_id: custom_type.id,
        name: "Custom Type Activity".to_string(),
        object_store_path: format!("test/{}.gpx", Uuid::new_v4()),
        started_at: now,
        submitted_at: now,
        visibility: Visibility::Public.as_str().to_string(),
        type_boundaries: None,
        segment_types: None,
    };

    db.save_activity(&activity)
        .await
        .expect("Failed to save activity with custom type");

    let retrieved = db
        .get_activity(activity.id)
        .await
        .expect("Failed to get activity")
        .expect("Activity not found");

    assert_eq!(retrieved.activity_type_id, custom_type.id);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_custom_type_in_multi_sport() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "custom_multi").await;

    // Use unique name to avoid conflicts
    let type_name = format!("dig_{}", Uuid::new_v4().simple());

    let custom_type = db
        .create_activity_type(&type_name, user_id)
        .await
        .expect("Failed to create custom type");

    let start = OffsetDateTime::now_utc();
    // MTB -> DIG -> MTB pattern (ride to trailwork, dig, ride home)
    let boundaries = vec![
        start,
        start + Duration::minutes(30),
        start + Duration::minutes(90),
        start + Duration::minutes(120),
    ];
    let segment_types = vec![builtin_types::MTB, custom_type.id, builtin_types::MTB];

    let activity =
        create_multi_sport_activity(&db, user_id, boundaries.clone(), segment_types.clone()).await;

    let retrieved = db
        .get_activity(activity.id)
        .await
        .expect("Failed to get activity")
        .expect("Activity not found");

    let stored_types = retrieved.segment_types.expect("Should have segment types");
    assert_eq!(stored_types[0], builtin_types::MTB);
    assert_eq!(stored_types[1], custom_type.id);
    assert_eq!(stored_types[2], builtin_types::MTB);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_list_activity_types_includes_custom() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "list_types").await;

    // Use unique name to avoid conflicts
    let type_name = format!("custom_list_test_{}", Uuid::new_v4().simple());

    // Create a custom type
    let custom_type = db
        .create_activity_type(&type_name, user_id)
        .await
        .expect("Failed to create custom type");

    // List all types
    let all_types = db
        .list_activity_types()
        .await
        .expect("Failed to list types");

    // Should include both built-in and custom
    let builtin_count = all_types.iter().filter(|t| t.is_builtin).count();
    let custom_found = all_types.iter().any(|t| t.id == custom_type.id);

    assert!(builtin_count >= 8, "Should have at least 8 built-in types");
    assert!(custom_found, "Custom type should be in list");

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_resolve_builtin_type_by_name() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());

    // Direct name match should work
    let resolved = db
        .resolve_activity_type("run")
        .await
        .expect("Failed to resolve type");

    match resolved {
        tracks::models::ResolvedActivityType::Exact(id) => {
            assert_eq!(id, builtin_types::RUN);
        }
        _ => panic!("Expected exact match for 'run'"),
    }
}

#[tokio::test]
async fn test_resolve_type_by_alias() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());

    // "running" is an alias for "run"
    let resolved = db
        .resolve_activity_type("running")
        .await
        .expect("Failed to resolve type");

    match resolved {
        tracks::models::ResolvedActivityType::Exact(id) => {
            assert_eq!(id, builtin_types::RUN);
        }
        _ => panic!("Expected exact match for 'running' alias"),
    }
}

#[tokio::test]
async fn test_resolve_ambiguous_alias() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());

    // "biking" is an ambiguous alias that maps to multiple types
    let resolved = db
        .resolve_activity_type("biking")
        .await
        .expect("Failed to resolve type");

    match resolved {
        tracks::models::ResolvedActivityType::Ambiguous(ids) => {
            assert!(ids.len() >= 2, "biking should map to multiple types");
        }
        _ => panic!("Expected ambiguous match for 'biking'"),
    }
}

// ============================================================================
// Test 9.4: Segment matching with multi-sport
// ============================================================================

#[tokio::test]
async fn test_segment_matching_single_sport() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "seg_single").await;

    // Create a RUN activity with track
    let activity = create_single_sport_activity(&db, user_id).await;

    // Create track points along a line
    let start_epoch = OffsetDateTime::now_utc().unix_timestamp();
    let points: Vec<(f64, f64, f64, i64)> = (0..100)
        .map(|i| {
            (
                40.0 + (i as f64 * 0.0001),   // lat
                -105.3 + (i as f64 * 0.0001), // lon
                1650.0 + (i as f64 * 0.5),    // elevation
                start_epoch + (i * 60),       // epoch seconds
            )
        })
        .collect();

    create_track_geometry(&pool, user_id, activity.id, &points).await;

    // Create a RUN segment that overlaps with the track
    let run_segment = create_test_segment(
        &pool,
        user_id,
        builtin_types::RUN,
        40.001,
        -105.299, // start
        40.003,
        -105.297, // end
    )
    .await;

    // Create an MTB segment at the same location (should NOT match)
    let mtb_segment = create_test_segment(
        &pool,
        user_id,
        builtin_types::MTB,
        40.001,
        -105.299,
        40.003,
        -105.297,
    )
    .await;

    // Find matching segments for RUN activity
    let matches = db
        .find_matching_segments(activity.id, builtin_types::RUN)
        .await
        .expect("Failed to find segments");

    // Should find the RUN segment but not the MTB segment
    assert!(
        matches.iter().any(|m| m.segment_id == run_segment),
        "Should find RUN segment"
    );
    assert!(
        !matches.iter().any(|m| m.segment_id == mtb_segment),
        "Should NOT find MTB segment"
    );

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_segment_matching_any_type() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "seg_any").await;

    // Create activity with track
    let activity = create_single_sport_activity(&db, user_id).await;

    let start_epoch = OffsetDateTime::now_utc().unix_timestamp();
    let points: Vec<(f64, f64, f64, i64)> = (0..100)
        .map(|i| {
            (
                40.0 + (i as f64 * 0.0001),
                -105.3 + (i as f64 * 0.0001),
                1650.0 + (i as f64 * 0.5),
                start_epoch + (i * 60),
            )
        })
        .collect();

    create_track_geometry(&pool, user_id, activity.id, &points).await;

    // Create segments of different types at the same location
    let run_segment = create_test_segment(
        &pool,
        user_id,
        builtin_types::RUN,
        40.001,
        -105.299,
        40.003,
        -105.297,
    )
    .await;

    let mtb_segment = create_test_segment(
        &pool,
        user_id,
        builtin_types::MTB,
        40.001,
        -105.299,
        40.003,
        -105.297,
    )
    .await;

    // find_matching_segments_any_type should find BOTH segments
    let matches = db
        .find_matching_segments_any_type(activity.id)
        .await
        .expect("Failed to find segments");

    assert!(
        matches.iter().any(|(m, _)| m.segment_id == run_segment),
        "Should find RUN segment"
    );
    assert!(
        matches.iter().any(|(m, _)| m.segment_id == mtb_segment),
        "Should find MTB segment"
    );

    // Verify the returned type IDs are correct
    let run_match = matches.iter().find(|(m, _)| m.segment_id == run_segment);
    let mtb_match = matches.iter().find(|(m, _)| m.segment_id == mtb_segment);

    assert_eq!(run_match.map(|(_, t)| *t), Some(builtin_types::RUN));
    assert_eq!(mtb_match.map(|(_, t)| *t), Some(builtin_types::MTB));

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_multi_sport_activity_data_for_filtering() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "multi_filter").await;

    // This test verifies that the multi-sport segment matching logic
    // correctly filters segments based on activity type at position.
    // The actual filtering happens in filter_multi_sport_matches in activity_queue.rs
    // which is tested in unit tests. Here we verify the database layer returns
    // the correct data to enable that filtering.

    let start = OffsetDateTime::now_utc();
    let boundaries = vec![
        start,
        start + Duration::minutes(50),
        start + Duration::minutes(100),
    ];
    let segment_types = vec![builtin_types::RUN, builtin_types::MTB];

    let activity =
        create_multi_sport_activity(&db, user_id, boundaries.clone(), segment_types.clone()).await;

    // Verify activity has the correct boundaries for caller to use
    let retrieved = db
        .get_activity(activity.id)
        .await
        .expect("Failed to get activity")
        .expect("Activity not found");

    assert!(retrieved.type_boundaries.is_some());
    assert!(retrieved.segment_types.is_some());

    let stored_boundaries = retrieved.type_boundaries.unwrap();
    let stored_types = retrieved.segment_types.unwrap();

    // Invariant: segment_types.len() == type_boundaries.len() - 1
    assert_eq!(
        stored_types.len(),
        stored_boundaries.len() - 1,
        "Invariant violated: segment_types.len() should equal type_boundaries.len() - 1"
    );

    cleanup_test_data(&pool, user_id).await;
}

// ============================================================================
// Additional edge case tests
// ============================================================================

#[tokio::test]
async fn test_activity_with_empty_arrays() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "empty_arrays").await;

    // Edge case: what happens with empty arrays?
    // This shouldn't happen in practice but let's verify behavior
    let now = OffsetDateTime::now_utc();
    let activity = Activity {
        id: Uuid::new_v4(),
        user_id,
        activity_type_id: builtin_types::RUN,
        name: "Empty Arrays Test".to_string(),
        object_store_path: format!("test/{}.gpx", Uuid::new_v4()),
        started_at: now,
        submitted_at: now,
        visibility: Visibility::Public.as_str().to_string(),
        type_boundaries: Some(vec![]), // Empty but present
        segment_types: Some(vec![]),   // Empty but present
    };

    db.save_activity(&activity)
        .await
        .expect("Failed to save activity");

    let retrieved = db
        .get_activity(activity.id)
        .await
        .expect("Failed to get activity")
        .expect("Activity not found");

    // Empty arrays should be stored and retrieved
    assert!(retrieved.type_boundaries.is_some());
    assert!(retrieved.segment_types.is_some());
    assert_eq!(retrieved.type_boundaries.as_ref().unwrap().len(), 0);
    assert_eq!(retrieved.segment_types.as_ref().unwrap().len(), 0);

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_get_user_activities_returns_started_at() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "started_at").await;

    // Create activity with a specific started_at time
    let started_at = OffsetDateTime::now_utc() - Duration::hours(2);
    let submitted_at = OffsetDateTime::now_utc();

    let activity = Activity {
        id: Uuid::new_v4(),
        user_id,
        activity_type_id: builtin_types::RUN,
        name: "Morning Run".to_string(),
        object_store_path: format!("test/{}.gpx", Uuid::new_v4()),
        started_at,
        submitted_at,
        visibility: Visibility::Public.as_str().to_string(),
        type_boundaries: None,
        segment_types: None,
    };

    db.save_activity(&activity)
        .await
        .expect("Failed to save activity");

    // get_user_activities must return started_at correctly
    // This test would fail if started_at is missing from the SELECT clause
    let activities = db
        .get_user_activities(user_id)
        .await
        .expect("Failed to get user activities");

    assert_eq!(activities.len(), 1);
    let retrieved = &activities[0];

    // Verify started_at matches (within 1 second to handle DB precision)
    let diff = (retrieved.started_at - started_at).abs();
    assert!(
        diff < Duration::seconds(1),
        "started_at mismatch: expected {started_at}, got {}",
        retrieved.started_at
    );

    // Also verify it's different from submitted_at
    assert_ne!(
        retrieved.started_at, retrieved.submitted_at,
        "started_at and submitted_at should be different"
    );

    cleanup_test_data(&pool, user_id).await;
}

#[tokio::test]
async fn test_user_activities_include_multi_sport_data() {
    let Some(pool) = get_test_pool().await else {
        return;
    };
    let db = Database::new(pool.clone());
    let user_id = create_test_user(&pool, "user_activities").await;

    // Create both single and multi-sport activities
    let single_sport = create_single_sport_activity(&db, user_id).await;

    let start = OffsetDateTime::now_utc();
    let multi_sport = create_multi_sport_activity(
        &db,
        user_id,
        vec![
            start,
            start + Duration::hours(1),
            start + Duration::hours(2),
        ],
        vec![builtin_types::RUN, builtin_types::MTB],
    )
    .await;

    // Get all user activities
    let activities = db
        .get_user_activities(user_id)
        .await
        .expect("Failed to get activities");

    assert_eq!(activities.len(), 2);

    // Find each activity and verify data
    let single = activities
        .iter()
        .find(|a| a.id == single_sport.id)
        .expect("Single-sport not found");
    let multi = activities
        .iter()
        .find(|a| a.id == multi_sport.id)
        .expect("Multi-sport not found");

    assert!(single.type_boundaries.is_none());
    assert!(single.segment_types.is_none());
    assert!(multi.type_boundaries.is_some());
    assert!(multi.segment_types.is_some());

    cleanup_test_data(&pool, user_id).await;
}
