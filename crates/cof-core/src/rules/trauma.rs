/// The Trauma consequence tiers (p.61). Effects are **cumulative**: a Broken character also
/// suffers everything from Shaken, Worn, and Battered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TraumaTier {
    /// 0–2: no lasting effects.
    Unscarred,
    /// 3–4: -1 Burden on Anguish Checks; Easy WIL check after a full rest or no Anguish lost.
    Shaken,
    /// 5–6: -1 Burden on STR and DEX checks; Carrying Capacity halved.
    Worn,
    /// 7–8: permanently Fatigued; old wounds tear open on the first Injury of a fight.
    Battered,
    /// 9–10: +1 Anguish whenever Anguish is gained; -2 Initiative.
    Broken,
    /// 11–12: Major Injury track reduced by 1; Average WIL check every morning or lose the day.
    Shattered,
    /// 13+: the character dies.
    TheEnd,
}

impl TraumaTier {
    pub fn for_score(score: u8) -> TraumaTier {
        match score {
            0..=2 => TraumaTier::Unscarred,
            3..=4 => TraumaTier::Shaken,
            5..=6 => TraumaTier::Worn,
            7..=8 => TraumaTier::Battered,
            9..=10 => TraumaTier::Broken,
            11..=12 => TraumaTier::Shattered,
            _ => TraumaTier::TheEnd,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            TraumaTier::Unscarred => "Unscarred",
            TraumaTier::Shaken => "Shaken",
            TraumaTier::Worn => "Worn",
            TraumaTier::Battered => "Battered",
            TraumaTier::Broken => "Broken",
            TraumaTier::Shattered => "Shattered",
            TraumaTier::TheEnd => "The End",
        }
    }
}

/// Trauma (p.60–61): the long-term toll of grievous wounds. Each Major Injury suffered in a
/// fight adds 1 at the end of it. Reaching 13 kills the character.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Trauma {
    current: u8,
}

/// The result of gaining Trauma.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TraumaGain {
    pub total: u8,
    pub from: TraumaTier,
    pub to: TraumaTier,
}

impl TraumaGain {
    /// True if this gain crossed into a new consequence tier.
    pub fn tier_changed(&self) -> bool {
        self.from != self.to
    }
}

impl Trauma {
    /// Trauma lost per week of uninterrupted rest in genuine safety (p.61).
    pub const REST_PER_WEEK: u8 = 5;
    /// Weekly rest with the Quick Recovery skill doubles the rate (p.61).
    pub const QUICK_RECOVERY_PER_WEEK: u8 = 10;
    /// A successful Hard healing check from a Healer, once a week (p.61).
    pub const DEDICATED_CARE: u8 = 2;
    /// A Glimmer of Peace: a successful Average WIL check in a rare cathartic moment (p.61).
    pub const GLIMMER_OF_PEACE: u8 = 2;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> u8 {
        self.current
    }

    pub fn tier(&self) -> TraumaTier {
        TraumaTier::for_score(self.current)
    }

    /// True once Trauma has reached 13 (p.61).
    pub fn is_fatal(&self) -> bool {
        self.tier() == TraumaTier::TheEnd
    }

    pub fn gain(&mut self, amount: u8) -> TraumaGain {
        let from = self.tier();
        self.current = self.current.saturating_add(amount);
        TraumaGain {
            total: self.current,
            from,
            to: self.tier(),
        }
    }

    /// Reduces Trauma. Has no effect once Trauma has proven fatal.
    pub fn heal(&mut self, amount: u8) {
        if !self.is_fatal() {
            self.current = self.current.saturating_sub(amount);
        }
    }

    fn at_least(&self, tier: TraumaTier) -> bool {
        self.tier() >= tier
    }

    /// Modifier to Anguish Check rolls: -1 Burden from Shaken onwards.
    pub fn anguish_check_modifier(&self) -> i8 {
        if self.at_least(TraumaTier::Shaken) {
            -1
        } else {
            0
        }
    }

    /// Burden on STR- and DEX-based checks: 1 from Worn onwards.
    pub fn str_dex_burden(&self) -> u8 {
        u8::from(self.at_least(TraumaTier::Worn))
    }

    /// Carrying Capacity after Trauma: halved (rounding down) from Worn onwards.
    pub fn carrying_capacity(&self, base: u8) -> u8 {
        if self.at_least(TraumaTier::Worn) {
            base / 2
        } else {
            base
        }
    }

    /// From Battered onwards the character is permanently Fatigued.
    pub fn permanently_fatigued(&self) -> bool {
        self.at_least(TraumaTier::Battered)
    }

    /// Extra Anguish gained whenever Anguish is gained: 1 from Broken onwards.
    pub fn extra_anguish(&self) -> u8 {
        u8::from(self.at_least(TraumaTier::Broken))
    }

    /// Initiative modifier: -2 from Broken onwards.
    pub fn initiative_modifier(&self) -> i8 {
        if self.at_least(TraumaTier::Broken) {
            -2
        } else {
            0
        }
    }

    /// Reduction to the Major Injury track: 1 from Shattered onwards.
    pub fn major_injury_track_reduction(&self) -> u8 {
        u8::from(self.at_least(TraumaTier::Shattered))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(score: u8) -> Trauma {
        let mut t = Trauma::new();
        t.gain(score);
        t
    }

    #[test]
    fn tier_boundaries_match_the_table() {
        let expected = [
            (0, TraumaTier::Unscarred),
            (2, TraumaTier::Unscarred),
            (3, TraumaTier::Shaken),
            (4, TraumaTier::Shaken),
            (5, TraumaTier::Worn),
            (7, TraumaTier::Battered),
            (9, TraumaTier::Broken),
            (10, TraumaTier::Broken),
            (11, TraumaTier::Shattered),
            (12, TraumaTier::Shattered),
            (13, TraumaTier::TheEnd),
            (40, TraumaTier::TheEnd),
        ];
        for (score, tier) in expected {
            assert_eq!(TraumaTier::for_score(score), tier, "score {score}");
        }
    }

    #[test]
    fn gain_reports_tier_changes() {
        let mut t = at(2);
        let gain = t.gain(1);
        assert_eq!(
            (gain.from, gain.to, gain.total),
            (TraumaTier::Unscarred, TraumaTier::Shaken, 3)
        );
        assert!(gain.tier_changed());
        assert!(!t.gain(1).tier_changed());
    }

    #[test]
    fn thirteen_is_fatal_and_final() {
        let mut t = at(12);
        assert!(!t.is_fatal());
        t.gain(1);
        assert!(t.is_fatal());
        t.heal(Trauma::REST_PER_WEEK);
        assert!(t.is_fatal());
    }

    #[test]
    fn healing_saturates_at_zero() {
        let mut t = at(4);
        t.heal(Trauma::QUICK_RECOVERY_PER_WEEK);
        assert_eq!(t.current(), 0);
    }

    #[test]
    fn effects_are_cumulative() {
        let unscarred = at(0);
        assert_eq!(unscarred.anguish_check_modifier(), 0);
        assert_eq!(unscarred.carrying_capacity(13), 13);

        let broken = at(9);
        assert_eq!(broken.anguish_check_modifier(), -1);
        assert_eq!(broken.str_dex_burden(), 1);
        assert_eq!(broken.carrying_capacity(13), 6);
        assert!(broken.permanently_fatigued());
        assert_eq!(broken.extra_anguish(), 1);
        assert_eq!(broken.initiative_modifier(), -2);
        assert_eq!(broken.major_injury_track_reduction(), 0);

        assert_eq!(at(11).major_injury_track_reduction(), 1);
    }

    #[test]
    fn worn_but_not_battered() {
        let worn = at(6);
        assert_eq!(worn.str_dex_burden(), 1);
        assert!(!worn.permanently_fatigued());
    }
}
