use std::fmt::Debug;

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
