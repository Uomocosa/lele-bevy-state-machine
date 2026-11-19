pub trait Clock: Copy + Send + Sync + 'static {
    fn now() -> Self;
}

#[cfg(test)]
mod tests {
    use crate::testing;
    use crate::traits::Clock;

    #[test]
    fn test_usage() {
        assert_eq!(*testing::FixedMoment::now(), 9);
    }
}
