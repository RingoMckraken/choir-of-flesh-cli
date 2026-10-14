use cof_core::rules::AnguishGain;

/// A journal line for gaining Anguish, e.g. "Anguish +2 → 5. Anguish Check D20 = 4 (+2) = 6:
/// holds."
pub fn describe(gain: &AnguishGain) -> String {
    let mut text = format!("Anguish +{} → {}.", gain.gained, gain.total);
    if let Some(check) = gain.check {
        let modifier = check.total - i16::from(check.natural);
        let roll = if modifier == 0 {
            format!("D20 = {}", check.natural)
        } else {
            format!("D20 = {} ({modifier:+}) = {}", check.natural, check.total)
        };
        let outcome = match check.breakdown {
            Some(b) => format!("BREAKS. Breakdown: {}!", b.name()),
            None => "holds… for now.".to_owned(),
        };
        text.push_str(&format!(" Anguish Check {roll}: {outcome}"));
    }
    text
}
