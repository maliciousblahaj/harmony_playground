#[derive(Debug, Clone)]
pub enum Error {
    MismatchedFrequencyIds {
        global_frequency_id: usize,
        relative_frequency_id: usize,
    },
    InvalidGlobalFrequencyId(usize),
}
