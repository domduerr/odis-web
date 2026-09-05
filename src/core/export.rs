use odis::FormalContext;

pub fn generate_cxt_string(ctx: &FormalContext<String>) -> String {
    let mut content = format!("B\n{}\n{}\n{}\n\n", ctx.name, ctx.objects.len(), ctx.attributes.len());

    for object in ctx.objects.iter() {
        if object != &"".to_string() {
            content.push_str(object);
        } else {
            content.push_str("\"no name\"");
        }
        content.push('\n');
    }
    for attribute in ctx.attributes.iter() {
        if attribute != &"".to_string() {
            content.push_str(attribute);
        } else {
            content.push_str("\"no name\"");
        }
        content.push('\n');
    }
    for column in 0..ctx.objects.len() {
        for row in 0..ctx.attributes.len() {
            if ctx.incidence.contains(&(column, row)) {
                content.push('X');
            } else {
                content.push('.');
            }
        }
        content.push('\n');
    }

    content
}

pub fn generate_cxt_filename(ctx: &FormalContext<String>) -> String {
    let name = ctx.name.trim();
    if name.is_empty() {
        "formal_context.cxt".to_string()
    } else {
        let safe: String = name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        format!("{}.cxt", safe)
    }
}
