/// Detect TPS drops below threshold
pub fn detect_tps_drops(tps_values: &[f64], threshold: f64) -> Vec<usize> {
    tps_values
        .iter()
        .enumerate()
        .filter_map(|(idx, &tps)| {
            if tps < threshold {
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

/// Detect if an entity is stuck based on position history
/// Returns true if entity moved less than threshold distance over all recorded positions
pub fn is_stuck_entity(positions: &[(f64, f64)], threshold: u32) -> bool {
    if positions.len() < 2 {
        return false;
    }

    let first = positions[0];
    let last = positions[positions.len() - 1];

    let distance = ((last.0 - first.0).powi(2) + (last.1 - first.1).powi(2)).sqrt();

    distance < threshold as f64
}

/// Detect consecutive position duplicates (completely stuck)
pub fn is_completely_stuck(positions: &[(f64, f64)], min_consecutive: usize) -> bool {
    if positions.len() < min_consecutive {
        return false;
    }

    let mut consecutive_count = 1;

    for i in 1..positions.len() {
        if positions[i] == positions[i - 1] {
            consecutive_count += 1;
            if consecutive_count >= min_consecutive {
                return true;
            }
        } else {
            consecutive_count = 1;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_single_tps_drop() {
        let tps = vec![60.0, 59.0, 5.0, 58.0];
        let drops = detect_tps_drops(&tps, 10.0);
        assert_eq!(drops, vec![2]);
    }

    #[test]
    fn test_detect_multiple_tps_drops() {
        let tps = vec![60.0, 5.0, 50.0, 8.0, 60.0];
        let drops = detect_tps_drops(&tps, 10.0);
        assert_eq!(drops, vec![1, 3]);
    }

    #[test]
    fn test_no_tps_drops() {
        let tps = vec![60.0, 59.0, 58.0, 60.0];
        let drops = detect_tps_drops(&tps, 10.0);
        let expected: Vec<usize> = vec![];
        assert_eq!(drops, expected);
    }

    #[test]
    fn test_stuck_entity_minimal_movement() {
        let positions = vec![
            (100.0, 200.0),
            (100.0, 200.0),
            (100.0, 200.0),
            (100.0, 200.0),
            (100.1, 200.1),
        ];
        assert!(is_stuck_entity(&positions, 50));
    }

    #[test]
    fn test_moving_entity() {
        let positions = vec![
            (100.0, 200.0),
            (110.0, 210.0),
            (120.0, 220.0),
            (130.0, 230.0),
        ];
        assert!(!is_stuck_entity(&positions, 10));
    }

    #[test]
    fn test_completely_stuck_entity() {
        let positions = vec![
            (100.0, 200.0),
            (100.0, 200.0),
            (100.0, 200.0),
            (100.0, 200.0),
        ];
        assert!(is_completely_stuck(&positions, 4));
    }

    #[test]
    fn test_not_completely_stuck() {
        let positions = vec![
            (100.0, 200.0),
            (100.0, 200.0),
            (101.0, 201.0),
            (100.0, 200.0),
        ];
        assert!(!is_completely_stuck(&positions, 3));
    }
}
