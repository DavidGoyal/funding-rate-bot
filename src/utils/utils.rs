#[derive(Debug, Clone, Copy)]
pub enum RoundingMode {
    Floor,
    Ceil,
    Round,
}

pub fn round_to_min_change_f64(
    value: f64,
    min_change: f64,
    rounding_mode: Option<RoundingMode>,
) -> f64 {
    let mode = rounding_mode.unwrap_or(RoundingMode::Round);

    // Divide value by min_change
    let divided = value / min_change;

    // Round to 0 decimal places based on rounding mode
    let rounded = match mode {
        RoundingMode::Floor => divided.floor(),
        RoundingMode::Ceil => divided.ceil(),
        RoundingMode::Round => divided.round(),
    };

    // Multiply back by min_change
    let result = rounded * min_change;

    // Get decimal places of min_change
    let decimal_places = get_decimal_places(min_change);

    // Round to the same decimal places as min_change
    round_to_decimal_places(result, decimal_places)
}

fn get_decimal_places(value: f64) -> u32 {
    let s = format!("{:.10}", value);
    let parts: Vec<&str> = s.split('.').collect();

    if parts.len() == 2 {
        // Count trailing zeros and remove them to get actual decimal places
        parts[1].trim_end_matches('0').len() as u32
    } else {
        0
    }
}

fn round_to_decimal_places(value: f64, decimal_places: u32) -> f64 {
    let multiplier = 10_f64.powi(decimal_places as i32);
    (value * multiplier).round() / multiplier
}
