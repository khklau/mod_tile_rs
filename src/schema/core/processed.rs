use std::fmt::Debug;
use std::option::Option;

#[derive(Debug)]
pub enum ProcessOutcome<R> {
    Processed(R),
    Ignored,
}

impl<R> ProcessOutcome<R> {
    pub fn is_processed(&self) -> bool {
        match self {
            ProcessOutcome::Processed(_) => true,
            ProcessOutcome::Ignored => false,
        }
    }

    pub fn processed<T>(self, other: T) -> Option<(R, T)> {
        match self {
            ProcessOutcome::Processed(result) => Some((result, other)),
            ProcessOutcome::Ignored => None,
        }
    }

    #[cfg(test)]
    pub fn is_ignored(&self) -> bool {
        match self {
            ProcessOutcome::Processed(_) => false,
            ProcessOutcome::Ignored => true,
        }
    }

    #[cfg(test)]
    pub fn expect_processed(self) -> R {
        if let ProcessOutcome::Processed(result) = self {
            return result;
        }
        panic!("Expected processed ProcessOutcome");
    }
}
