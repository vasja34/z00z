pub fn assert_absent(label: &str, source: &str, needle: &str) {
    assert!(
        !source.contains(needle),
        "{} must not contain forbidden source shape {:?}",
        label,
        needle
    );
}

pub fn assert_all_absent(label: &str, source: &str, needles: &[&str]) {
    for needle in needles {
        assert_absent(label, source, needle);
    }
}

pub fn assert_present(label: &str, source: &str, needle: &str) {
    assert!(
        source.contains(needle),
        "{} must contain {:?}",
        label,
        needle
    );
}

pub fn assert_all_present(label: &str, source: &str, needles: &[&str]) {
    for needle in needles {
        assert_present(label, source, needle);
    }
}

pub fn assert_each_absent(sources: &[(&str, &str)], needles: &[&str]) {
    for (label, source) in sources {
        assert_all_absent(label, source, needles);
    }
}
