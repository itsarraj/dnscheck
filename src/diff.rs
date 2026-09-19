/// Whether every resolver that answered returned the same record set.
/// A resolver that failed to answer at all (`Err`) doesn't participate
/// in this comparison — it's reported separately as its own problem,
/// distinct from "the resolvers disagree with each other".
pub fn all_agree<'a>(record_sets: impl Iterator<Item = &'a [String]>) -> bool {
    let mut sets = record_sets;
    let Some(first) = sets.next() else {
        return true;
    };
    sets.all(|s| s == first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_sets_agree() {
        let a = vec!["1.2.3.4".to_string()];
        let b = vec!["1.2.3.4".to_string()];
        assert!(all_agree([a.as_slice(), b.as_slice()].into_iter()));
    }

    #[test]
    fn differing_sets_disagree() {
        let a = vec!["1.2.3.4".to_string()];
        let b = vec!["5.6.7.8".to_string()];
        assert!(!all_agree([a.as_slice(), b.as_slice()].into_iter()));
    }

    #[test]
    fn a_single_set_trivially_agrees_with_itself() {
        let a = vec!["1.2.3.4".to_string()];
        assert!(all_agree([a.as_slice()].into_iter()));
    }

    #[test]
    fn no_sets_at_all_is_vacuously_agreeing() {
        let empty: Vec<&[String]> = vec![];
        assert!(all_agree(empty.into_iter()));
    }

    #[test]
    fn multi_value_sets_compare_order_sensitively_so_callers_must_sort_first() {
        let a = vec!["1.1.1.1".to_string(), "2.2.2.2".to_string()];
        let b = vec!["2.2.2.2".to_string(), "1.1.1.1".to_string()];
        assert!(
            !all_agree([a.as_slice(), b.as_slice()].into_iter()),
            "this documents that all_agree does a plain Vec equality check — sorting happens in query.rs before results ever reach here"
        );
    }
}
