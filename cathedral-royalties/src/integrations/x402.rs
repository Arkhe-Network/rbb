pub struct X402RoyaltyServer {
    facilitator_url: String,
}

impl X402RoyaltyServer {
    pub fn new(facilitator_url: &str) -> Self {
        Self {
            facilitator_url: facilitator_url.to_string(),
        }
    }
}
