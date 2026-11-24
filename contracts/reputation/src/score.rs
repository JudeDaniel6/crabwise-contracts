//! Pure reputation-aggregation helpers (closes #974).
//!
//! This module is deliberately free of `Env` and any host access, mirroring the
//! pure/impure split in `contracts/campaign/src/require.rs`. That makes the
//! arithmetic — the part most likely to be wrong — exhaustively unit-testable
//! without a Soroban host.
//!
//! ## Cost model
//!
//! `get_score` reads a single maintained [`ReviewAggregate`](crate::types::ReviewAggregate)
//! entry and projects it here. It never scans the review set, so its cost is
//! bounded and constant in the number of reviews a subject has ever received.
//!
//! ## Relationship to `search::set_rating`
//!
//! `search` stores a caller-supplied `rating` per artist for ranking. It is a
//! *ranking hint*; this contract is the **authoritative reputation source**,
//! derived only from reviews that went through the moderation and appeal
//! lifecycle. A client should display the value returned by `get_score` and
//! treat `search`'s rating as a private ranking input. Unifying the two would
//! mean `search` reading this contract; that is a maintainer decision and is
//! called out in the PR.

use crate::types::{ReputationScore, ReviewAggregate};

/// Neutral prior rating, in tenths: `3.0`.
pub const PRIOR_RATING_X10: u32 = 30;
/// Weight of the prior, counted in "virtual reviews". A subject needs several
/// real reviews before its own average dominates the displayed score.
pub const PRIOR_WEIGHT: u32 = 3;

/// Lowest rating the contract accepts, in tenths: `1.0` (`submit_review`
/// validates this inclusive range).
pub const MIN_RATING_X10: u32 = 10;

/// Highest rating the contract accepts, in tenths: `5.0`.
pub const MAX_RATING_X10: u32 = 50;

/// Whether `rating_x10` is inside the range the contract accepts.
pub fn is_accepted_rating(rating_x10: u32) -> bool {
    (MIN_RATING_X10..=MAX_RATING_X10).contains(&rating_x10)
}

/// Mean rating in tenths over the active reviews; `0` when there are none.
///
/// Uses checked division so a zero `active_reviews` is handled explicitly
/// rather than as a division by zero. The result is clamped to
/// [`MAX_RATING_X10`]: `rating_sum` and `active_reviews` are stored separately,
/// and a removal lowers the count and the sum in the same call, so any drift
/// must not surface as a rating above the accepted ceiling. A quotient that
/// falls below [`MIN_RATING_X10`] (an inconsistent aggregate) is reported as
/// `0`, meaning "no usable rating", rather than as a sub-minimum score.
pub fn average_rating_x10(rating_sum: u32, active_reviews: u32) -> u32 {
    match rating_sum.checked_div(active_reviews) {
        Some(mean) if mean >= MIN_RATING_X10 => mean.min(MAX_RATING_X10),
        _ => 0,
    }
}

/// Confidence-weighted rating in tenths.
///
/// Blends the subject's own average toward [`PRIOR_RATING_X10`] with weight
/// [`PRIOR_WEIGHT`], so a brand-new subject with one five-star review does not
/// display a perfect score. Every step is checked or saturating and the result
/// is clamped to [`MAX_RATING_X10`], so no input pair can report above `5.0`.
pub fn weighted_score_x10(rating_sum: u32, active_reviews: u32) -> u32 {
    let prior_total = PRIOR_RATING_X10
        .checked_mul(PRIOR_WEIGHT)
        .unwrap_or(u32::MAX);
    let numerator = rating_sum.saturating_add(prior_total);
    // `PRIOR_WEIGHT` is a positive constant, so the denominator can never be
    // zero; fall back to the prior rather than panicking on absurd inputs.
    let denominator = active_reviews.saturating_add(PRIOR_WEIGHT);
    numerator
        .checked_div(denominator)
        .unwrap_or(PRIOR_RATING_X10)
        .min(MAX_RATING_X10)
}

/// Project a stored aggregate into the client-facing score.
pub fn score_from_aggregate(aggregate: &ReviewAggregate) -> ReputationScore {
    ReputationScore {
        subject: aggregate.subject.clone(),
        total_reviews: aggregate.total_reviews,
        active_reviews: aggregate.active_reviews,
        average_rating_x10: average_rating_x10(aggregate.rating_sum, aggregate.active_reviews),
        rating_sum: aggregate.rating_sum,
        weighted_score_x10: weighted_score_x10(aggregate.rating_sum, aggregate.active_reviews),
        removed_reviews: aggregate.removed_reviews,
        appeal_count: aggregate.appeal_count,
        moderation_count: aggregate.moderation_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn aggregate(env: &Env) -> ReviewAggregate {
        ReviewAggregate {
            subject: Address::generate(env),
            total_reviews: 0,
            active_reviews: 0,
            rating_sum: 0,
            removed_reviews: 0,
            appeal_count: 0,
            moderation_count: 0,
        }
    }

    #[test]
    fn average_of_no_reviews_is_zero() {
        assert_eq!(average_rating_x10(0, 0), 0);
    }

    #[test]
    fn accepted_rating_bounds_are_inclusive() {
        assert!(is_accepted_rating(MIN_RATING_X10));
        assert!(is_accepted_rating(MAX_RATING_X10));
        assert!(!is_accepted_rating(MIN_RATING_X10 - 1));
        assert!(!is_accepted_rating(MAX_RATING_X10 + 1));
    }

    #[test]
    fn average_of_the_lowest_and_highest_ratings_is_exact() {
        assert_eq!(average_rating_x10(MIN_RATING_X10, 1), MIN_RATING_X10);
        assert_eq!(average_rating_x10(MAX_RATING_X10, 1), MAX_RATING_X10);
        assert_eq!(average_rating_x10(60, 2), 30);
    }

    #[test]
    fn an_inconsistent_aggregate_reports_no_rating() {
        // A positive count with a sum below the minimum cannot describe a real
        // review: report `0` (no usable rating), not a sub-minimum score.
        assert_eq!(average_rating_x10(0, 1), 0);
        assert_eq!(average_rating_x10(MIN_RATING_X10 - 1, 1), 0);
    }

    #[test]
    fn average_is_the_integer_mean_in_tenths() {
        // Two reviews of 5.0 and 3.0 → 4.0 (40 tenths), no remainder.
        assert_eq!(average_rating_x10(80, 2), 40);
        // Truncation is toward zero, not rounded: 41 / 2 == 20.
        assert_eq!(average_rating_x10(41, 2), 20);
    }

    #[test]
    fn weighted_score_pulls_toward_the_neutral_prior() {
        // One perfect review: (50 + 30*3) / (1 + 3) = 140 / 4 = 35 (3.5).
        assert_eq!(weighted_score_x10(50, 1), 35);
        // No reviews at all: the prior itself, (0 + 90) / 3 = 30 (3.0).
        assert_eq!(weighted_score_x10(0, 0), 30);
    }

    #[test]
    fn weighted_score_approaches_the_true_average_as_reviews_accumulate() {
        // Ten perfect reviews: (500 + 90) / (10 + 3) = 590 / 13 = 45.
        let score = weighted_score_x10(500, 10);
        assert!(score > 40 && score < 50, "score {score} should sit near 5.0");
    }

    #[test]
    fn weighted_score_never_exceeds_the_rating_ceiling() {
        for sum in [0u32, 50, 500, 5_000, 50_000] {
            for count in [0u32, 1, 10, 100, 1_000] {
                let score = weighted_score_x10(sum, count);
                assert!(score <= 50, "score {score} out of range for {sum}/{count}");
            }
        }
    }

    #[test]
    fn scores_are_saturating_on_absurd_inputs() {
        let _ = weighted_score_x10(u32::MAX, u32::MAX);
        let _ = average_rating_x10(u32::MAX, 1);
    }

    #[test]
    fn score_from_aggregate_exposes_every_component() {
        let env = Env::default();
        let mut agg = aggregate(&env);
        agg.total_reviews = 4;
        agg.active_reviews = 2;
        agg.rating_sum = 80;
        agg.removed_reviews = 2;
        agg.appeal_count = 1;
        agg.moderation_count = 3;

        let score = score_from_aggregate(&agg);
        assert_eq!(score.subject, agg.subject);
        assert_eq!(score.total_reviews, 4);
        assert_eq!(score.active_reviews, 2);
        assert_eq!(score.average_rating_x10, 40);
        assert_eq!(score.rating_sum, 80);
        assert_eq!(score.removed_reviews, 2);
        assert_eq!(score.appeal_count, 1);
        assert_eq!(score.moderation_count, 3);
    }
}
