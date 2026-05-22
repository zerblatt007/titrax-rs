/// Produces a sort key for Norwegian alphabetical order.
/// Standard Latin letters sort normally; Æ sorts after Z, Ø after Æ, Å after Ø.
pub fn norwegian_sort_key(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'Æ' | 'æ' => '\u{007E}', // ~ (after Z in ASCII)
            'Ø' | 'ø' => '\u{007F}',
            'Å' | 'å' => '\u{0080}',
            other => other.to_ascii_lowercase(),
        })
        .collect()
}

pub fn sort_projects(projects: &mut Vec<crate::app::Project>) {
    projects.sort_by(|a, b| {
        norwegian_sort_key(&a.name).cmp(&norwegian_sort_key(&b.name))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_norwegian_sort_order() {
        let mut names = vec!["Åse", "Øyvind", "Æsop", "Zebra", "Anna"];
        names.sort_by(|a, b| norwegian_sort_key(a).cmp(&norwegian_sort_key(b)));
        assert_eq!(names, vec!["Anna", "Zebra", "Æsop", "Øyvind", "Åse"]);
    }

    #[test]
    fn test_case_insensitive_sort() {
        let mut names = vec!["beta", "Alpha", "gamma"];
        names.sort_by(|a, b| norwegian_sort_key(a).cmp(&norwegian_sort_key(b)));
        assert_eq!(names, vec!["Alpha", "beta", "gamma"]);
    }
}
