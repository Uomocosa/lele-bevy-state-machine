use derive_more::Deref;

use crate::traits::Clock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct FixedMoment(pub u64);

#[rustfmt::skip]
impl Clock for FixedMoment {
    fn now() -> Self { Self(9) }
}

#[cfg(test)]
mod tests {
    use crate::traits::Clock;

    use super::FixedMoment;

    #[test]
    fn test_usage() {
        assert_eq!(*FixedMoment::now(), 9);
    }
}
