use bit_set::BitSet;

#[allow(dead_code)]
pub fn format_object_set(indices: &BitSet, names: &[String]) -> String {
    let mut obj_string = String::new();
    obj_string.push('{');

    for n in indices {
        if n < names.len() {
            obj_string.push_str(&format!(" {} ,", names[n]));
        }
    }

    if !indices.is_empty() {
        obj_string.pop();
    } else {
        obj_string.push(' ');
    }
    obj_string.push('}');
    obj_string
}

pub fn format_attribute_set(indices: &BitSet, names: &[String]) -> String {
    let mut attr_string = String::new();
    attr_string.push('{');

    for n in indices {
        if n < names.len() {
            attr_string.push_str(&format!(" {} ,", names[n]));
        }
    }

    if !indices.is_empty() {
        attr_string.pop();
    } else {
        attr_string.push(' ');
    }
    attr_string.push('}');
    attr_string
}

