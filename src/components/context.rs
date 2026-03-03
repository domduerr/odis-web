use bit_set::BitSet;
use odis::FormalContext;

pub fn create_default_context() -> FormalContext<String> {
    let mut ctx = FormalContext::new();
    for n in 0..5 {
        ctx.add_object(format!("{}", n + 1), &BitSet::new());
        ctx.add_attribute(format!("{}", (b'a' + n as u8) as char), &BitSet::new());
    }
    ctx
}

pub fn index_to_column_name(index: usize) -> String {
    let mut result = String::new();
    let mut n = index;
    loop {
        result.push((b'a' + (n % 26) as u8) as char);
        n = n / 26;
        if n == 0 {
            break;
        }
        n -= 1;
    }
    result.chars().rev().collect()
}
