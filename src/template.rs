use std::collections::HashMap;

pub fn template_string(template_str: &str, context: &HashMap<&str, &str>) -> String {
    let mut ret = template_str.to_owned();

    for (key, val) in context.iter() {
        let pattern = format!("{{{{{key}}}}}");
        let match_ranges = ret
            .match_indices(&pattern)
            .map(|(i, m)| i..(i + m.len()))
            .collect::<Vec<_>>();
        for range in match_ranges {
            ret.replace_range(range, val);
        }
    }

    ret
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::template::template_string;

    #[test]
    fn test_simple_template() {
        let context = HashMap::from([("a", "Hello"), ("b", "World")]);

        let result = template_string("{{a}}, {{b}}!", &context);

        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_template_replaces_all_instances() {
        let context = HashMap::from([("a", "Hello"), ("b", "World")]);

        let result = template_string("{{a}}? {{a}}!", &context);

        assert_eq!(result, "Hello? Hello!");
    }

    #[test]
    fn test_ignores_nonexistent_template() {
        let context = HashMap::from([("a", "Hello"), ("b", "World")]);

        let result = template_string("{{a}}? {{c}}!", &context);

        assert_eq!(result, "Hello? {{c}}!");
    }
}
