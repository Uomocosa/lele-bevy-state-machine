pub trait Clock: Copy + Send + Sync + 'static {
    fn now() -> Self;
}

#[cfg(test)]
mod tests {
    use derive_more::Deref;

    use crate::clock::Clock;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Deref)]
    struct FixedMoment(u64);

    impl Clock for FixedMoment {
        fn now() -> Self {
            Self(9)
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(FixedMoment::now(), FixedMoment(9));
    }
}
