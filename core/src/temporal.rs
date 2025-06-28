//! Temporal utilities and helpers for working with bitemporal data

use chrono::{DateTime, Utc};
use crate::types::TimeEdge;

/// Utilities for working with temporal data
pub struct TemporalUtils;

impl TemporalUtils {
    /// Check if two time intervals overlap
    pub fn intervals_overlap(
        start1: DateTime<Utc>,
        end1: Option<DateTime<Utc>>,
        start2: DateTime<Utc>,
        end2: Option<DateTime<Utc>>,
    ) -> bool {
        let end1 = end1.unwrap_or_else(|| DateTime::<Utc>::MAX_UTC);
        let end2 = end2.unwrap_or_else(|| DateTime::<Utc>::MAX_UTC);
        
        start1 < end2 && start2 < end1
    }
    
    /// Check if a time point falls within an interval
    pub fn point_in_interval(
        point: DateTime<Utc>,
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
    ) -> bool {
        let end = end.unwrap_or_else(|| DateTime::<Utc>::MAX_UTC);
        point >= start && point < end
    }
    
    /// Get the current timestamp
    pub fn now() -> DateTime<Utc> {
        Utc::now()
    }
    
    /// Create a "current" edge that is valid from now with no end time
    pub fn create_current_edge<P>(
        from_node_id: uuid::Uuid,
        to_node_id: uuid::Uuid,
        kind: impl Into<String>,
        props: P,
    ) -> TimeEdge<P> {
        TimeEdge::new(from_node_id, to_node_id, kind, Self::now(), props)
    }
}

/// Allen's Interval Algebra relations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalRelation {
    Before,
    Meets,
    Overlaps,
    FinishedBy,
    Contains,
    Starts,
    Equals,
    StartedBy,
    During,
    Finishes,
    OverlappedBy,
    MetBy,
    After,
}

impl IntervalRelation {
    /// Determine the relation between two intervals
    pub fn determine(
        start1: DateTime<Utc>,
        end1: Option<DateTime<Utc>>,
        start2: DateTime<Utc>,
        end2: Option<DateTime<Utc>>,
    ) -> Self {
        let end1 = end1.unwrap_or_else(|| DateTime::<Utc>::MAX_UTC);
        let end2 = end2.unwrap_or_else(|| DateTime::<Utc>::MAX_UTC);
        
        if end1 < start2 {
            IntervalRelation::Before
        } else if end1 == start2 {
            IntervalRelation::Meets
        } else if start1 < start2 && end1 < end2 && end1 > start2 {
            IntervalRelation::Overlaps
        } else if start1 < start2 && end1 == end2 {
            IntervalRelation::FinishedBy
        } else if start1 < start2 && end1 > end2 {
            IntervalRelation::Contains
        } else if start1 == start2 && end1 < end2 {
            IntervalRelation::Starts
        } else if start1 == start2 && end1 == end2 {
            IntervalRelation::Equals
        } else if start1 == start2 && end1 > end2 {
            IntervalRelation::StartedBy
        } else if start1 > start2 && end1 < end2 {
            IntervalRelation::During
        } else if start1 > start2 && end1 == end2 {
            IntervalRelation::Finishes
        } else if start1 < end2 && start1 > start2 && end1 > end2 {
            IntervalRelation::OverlappedBy
        } else if start1 == end2 {
            IntervalRelation::MetBy
        } else {
            IntervalRelation::After
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_intervals_overlap() {
        let t1 = Utc.ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let t2 = Utc.ymd_opt(2024, 1, 2).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let t3 = Utc.ymd_opt(2024, 1, 3).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let t4 = Utc.ymd_opt(2024, 1, 4).unwrap().and_hms_opt(0, 0, 0).unwrap();
        
        // Non-overlapping intervals
        assert!(!TemporalUtils::intervals_overlap(t1, Some(t2), t3, Some(t4)));
        
        // Overlapping intervals
        assert!(TemporalUtils::intervals_overlap(t1, Some(t3), t2, Some(t4)));
        
        // Open-ended interval
        assert!(TemporalUtils::intervals_overlap(t1, None, t2, Some(t4)));
    }
    
    #[test]
    fn test_point_in_interval() {
        let start = Utc.ymd_opt(2024, 1, 1).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let middle = Utc.ymd_opt(2024, 1, 2).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let end = Utc.ymd_opt(2024, 1, 3).unwrap().and_hms_opt(0, 0, 0).unwrap();
        
        assert!(TemporalUtils::point_in_interval(middle, start, Some(end)));
        assert!(!TemporalUtils::point_in_interval(end, start, Some(end))); // End is exclusive
        assert!(TemporalUtils::point_in_interval(middle, start, None)); // Open interval
    }
}