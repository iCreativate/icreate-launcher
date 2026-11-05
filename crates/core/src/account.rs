//! Optional iCreate accounts. Missing or stale data never blocks installing or opening apps.

use serde::{Deserialize, Serialize};

/// iCreate Pro in South African rand.
pub const PRO_MONTHLY_ZAR: &str = "99.00";
/// Yearly iCreate Pro (ten months' price).
pub const PRO_YEARLY_ZAR: &str = "990.00";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Plan {
    #[default]
    Free,
    Pro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingInterval {
    Month,
    Year,
}

impl BillingInterval {
    pub fn label(self) -> &'static str {
        match self {
            Self::Month => "month",
            Self::Year => "year",
        }
    }

    pub fn amount_zar(self) -> &'static str {
        match self {
            Self::Month => PRO_MONTHLY_ZAR,
            Self::Year => PRO_YEARLY_ZAR,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    #[default]
    Inactive,
    Active,
    Cancelled,
    PastDue,
}

/// What the UI may show for an optional account. Safe to keep on disk (no tokens).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AccountSnapshot {
    /// Supabase URL and anon key are present, so sign-in can be offered.
    pub configured: bool,
    /// A sign-in, refresh or checkout is in flight.
    pub busy: bool,
    pub user_id: Option<String>,
    pub email: Option<String>,
    pub plan: Plan,
    pub interval: Option<BillingInterval>,
    pub status: SubscriptionStatus,
    /// Unix seconds; `None` means "no known end" (Free, or Pro until ITN fills it in).
    pub period_end: Option<i64>,
    /// Last network error, if any. Apps still run.
    pub last_error: Option<String>,
}

impl AccountSnapshot {
    pub fn signed_in(&self) -> bool {
        self.user_id.is_some()
    }

    /// Pro extras only. Install, update and launch never consult this.
    pub fn is_pro(&self, now: i64) -> bool {
        if self.plan != Plan::Pro {
            return false;
        }
        match self.status {
            SubscriptionStatus::Active => self.period_end.is_none_or(|end| end > now),
            SubscriptionStatus::Cancelled => self.period_end.is_some_and(|end| end > now),
            SubscriptionStatus::Inactive | SubscriptionStatus::PastDue => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsigned_and_free_are_never_pro() {
        let snap = AccountSnapshot::default();
        assert!(!snap.signed_in());
        assert!(!snap.is_pro(1));
        let free = AccountSnapshot { user_id: Some("u".into()), plan: Plan::Free, status: SubscriptionStatus::Active, ..AccountSnapshot::default() };
        assert!(free.signed_in());
        assert!(!free.is_pro(1));
    }

    #[test]
    fn pro_stays_active_until_period_end_after_cancel() {
        let now = 1_000;
        let active =
            AccountSnapshot { plan: Plan::Pro, status: SubscriptionStatus::Active, period_end: Some(now + 10), ..AccountSnapshot::default() };
        assert!(active.is_pro(now));
        assert!(!active.is_pro(now + 10));
        let cancelled = AccountSnapshot { status: SubscriptionStatus::Cancelled, ..active.clone() };
        assert!(cancelled.is_pro(now));
        assert!(!cancelled.is_pro(now + 10));
        let past_due = AccountSnapshot { status: SubscriptionStatus::PastDue, period_end: Some(now + 10), ..active };
        assert!(!past_due.is_pro(now));
    }
}
