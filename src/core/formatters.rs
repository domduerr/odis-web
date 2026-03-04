use bit_set::BitSet;

pub fn format_object_set(indices: &BitSet, names: &[String]) -> String {
    let mut obj_string = String::new();
    obj_string.push('{');

    for n in indices {
        if n < names.len() {
            obj_string.push_str(&format!(" {} ,", names[n]));
        }
    }

    if indices.len() > 0 {
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

    if indices.len() > 0 {
        attr_string.pop();
    } else {
        attr_string.push(' ');
    }
    attr_string.push('}');
    attr_string
}

pub fn format_concept_set(indices: &BitSet, names: &[String], index: usize) -> (String, String) {
    let set_str = format_object_set(indices, names);

    let white_spaces = if index >= 9 {
        String::from("   ")
    } else {
        String::from("   ")
    };

    (
        format!("{}:{}{},", index + 1, white_spaces, set_str),
        set_str,
    )
}

pub fn format_implication(
    premise: &BitSet,
    conclusion: &BitSet,
    attr_names: &[String],
    index: usize,
) -> (String, String) {
    let premise_str = format_attribute_set(premise, attr_names);
    let conclusion_str = format_attribute_set(conclusion, attr_names);

    let white_spaces = if index >= 9 {
        String::from("   ")
    } else {
        String::from("   ")
    };

    (
        format!("{}:{}{}", index + 1, white_spaces, premise_str),
        format!("{}{}", String::from("      "), conclusion_str),
    )
}
