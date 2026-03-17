use anyhow::{bail, Context, Result};

pub fn parse_relativistic_fraction(input: &str) -> Result<f64> {
    let value = input.trim();
    if !value.ends_with('c') {
        bail!("relativistic must end with 'c', for example 0.8c");
    }

    let number = &value[..value.len() - 1];
    let beta: f64 = number
        .parse()
        .with_context(|| format!("invalid relativistic value: {input}"))?;

    if !(0.0..1.0).contains(&beta) && beta != 0.0 {
        bail!("relativistic fraction must be in [0.0, 1.0)");
    }

    Ok(beta)
}

pub fn parse_kelvin(input: &str) -> Result<f64> {
    let value = input.trim();
    if !value.ends_with('K') {
        bail!("target-temp must end with 'K', for example 77K");
    }

    let number = &value[..value.len() - 1];
    let kelvin: f64 = number
        .parse()
        .with_context(|| format!("invalid Kelvin value: {input}"))?;

    if kelvin < 0.0 {
        bail!("target temperature cannot be negative");
    }

    Ok(kelvin)
}

#[cfg(test)]
mod tests {
    use super::{parse_kelvin, parse_relativistic_fraction};

    #[test]
    fn parses_relativistic_fraction() {
        let beta = parse_relativistic_fraction("0.8c").expect("beta should parse");
        assert!((beta - 0.8).abs() < 1e-12);
    }

    #[test]
    fn parses_kelvin() {
        let kelvin = parse_kelvin("77K").expect("kelvin should parse");
        assert!((kelvin - 77.0).abs() < 1e-12);
    }
}
