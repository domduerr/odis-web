use bit_set::BitSet;

/// Renders a set of object or attribute indices as `{a, b, c}`.
///
/// The single formatter for every set the UI shows, so that concepts,
/// implications and exploration questions all read the same.
pub fn format_set(indices: &BitSet, names: &[String]) -> String {
    let items: Vec<&str> = indices
        .iter()
        .filter(|&n| n < names.len())
        .map(|n| names[n].as_str())
        .collect();

    format!("{{{}}}", items.join(", "))
}

/// `1 concept`, `2 concepts` — every count the UI shows goes through here.
pub fn count(n: usize, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

/// The size of a context, as shown next to its name: `6 objects × 12 attributes`.
pub fn context_size(objects: usize, attributes: usize) -> String {
    format!("{} × {}", count(objects, "object"), count(attributes, "attribute"))
}
