/// Heuristics for resolving symbols to global nodes.
/// Ports the exact ResolutionTier and confidence scoring from original GitNexus.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FallbackReason {
    ImplicitSelf,
    VueComponent,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolutionTier {
    SameFile,
    ImportScoped,
    Global,
    Fallback(FallbackReason),
}

impl ResolutionTier {
    /// Returns the base confidence score for the resolution tier.
    pub fn base_confidence(&self) -> f32 {
        match self {
            ResolutionTier::SameFile => 1.0,
            ResolutionTier::ImportScoped => 0.95,
            ResolutionTier::Global => 0.7,
            ResolutionTier::Fallback(reason) => match reason {
                FallbackReason::ImplicitSelf => 0.8,
                FallbackReason::VueComponent => 0.8,
                FallbackReason::Unknown => 0.4,
            },
        }
    }
}
