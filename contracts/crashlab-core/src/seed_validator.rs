use crate::CaseSeed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedSchema {
    pub min_payload_len: usize,
    pub max_payload_len: usize,
    pub min_id: u64,
    pub max_id: u64,
}

impl Default for SeedSchema {
    fn default() -> Self {
        Self {
            min_payload_len: 1,
            max_payload_len: 64,
            min_id: 0,
            max_id: u64::MAX,
        }
    }
}

impl SeedSchema {
    pub fn new(min_payload_len: usize, max_payload_len: usize, min_id: u64, max_id: u64) -> Self {
        Self {
            min_payload_len,
            max_payload_len,
            min_id,
            max_id,
        }
    }

    pub fn with_payload_bounds(min: usize, max: usize) -> Self {
        Self::new(min, max, 0, u64::MAX)
    }

    pub fn with_id_bounds(min: u64, max: u64) -> Self {
        Self::new(0, 64, min, max)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedValidationError {
    PayloadTooShort { actual: usize, minimum: usize },
    PayloadTooLong { actual: usize, maximum: usize },
    IdTooSmall { actual: u64, minimum: u64 },
    IdTooLarge { actual: u64, maximum: u64 },
}

impl std::fmt::Display for SeedValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeedValidationError::PayloadTooShort { actual, minimum } => {
                write!(
                    f,
                    "payload too short: {} bytes, minimum {}",
                    actual, minimum
                )
            }
            SeedValidationError::PayloadTooLong { actual, maximum } => {
                write!(f, "payload too long: {} bytes, maximum {}", actual, maximum)
            }
            SeedValidationError::IdTooSmall { actual, minimum } => {
                write!(f, "id too small: {}, minimum {}", actual, minimum)
            }
            SeedValidationError::IdTooLarge { actual, maximum } => {
                write!(f, "id too large: {}, maximum {}", actual, maximum)
            }
        }
    }
}

pub trait Validate {
    fn validate(&self, schema: &SeedSchema) -> Result<(), Vec<SeedValidationError>>;
}

impl Validate for CaseSeed {
    fn validate(&self, schema: &SeedSchema) -> Result<(), Vec<SeedValidationError>> {
        let mut errors = Vec::new();

        if self.payload.len() < schema.min_payload_len {
            errors.push(SeedValidationError::PayloadTooShort {
                actual: self.payload.len(),
                minimum: schema.min_payload_len,
            });
        }

        if self.payload.len() > schema.max_payload_len {
            errors.push(SeedValidationError::PayloadTooLong {
                actual: self.payload.len(),
                maximum: schema.max_payload_len,
            });
        }

        if self.id < schema.min_id {
            errors.push(SeedValidationError::IdTooSmall {
                actual: self.id,
                minimum: schema.min_id,
            });
        }

        if self.id > schema.max_id {
            errors.push(SeedValidationError::IdTooLarge {
                actual: self.id,
                maximum: schema.max_id,
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn validate_seed(seed: &CaseSeed) -> Result<(), Vec<SeedValidationError>> {
    seed.validate(&SeedSchema::default())
}

pub fn validate_seed_with_schema(
    seed: &CaseSeed,
    schema: &SeedSchema,
) -> Result<(), Vec<SeedValidationError>> {
    seed.validate(schema)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_schema_accepts_valid_seed() {
        let seed = CaseSeed {
            id: 42,
            payload: vec![1, 2, 3, 4],
        };
        assert!(seed.validate(&SeedSchema::default()).is_ok());
    }

    #[test]
    fn default_schema_rejects_empty_payload() {
        let seed = CaseSeed {
            id: 1,
            payload: vec![],
        };
        let result = seed.validate(&SeedSchema::default());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&SeedValidationError::PayloadTooShort {
            actual: 0,
            minimum: 1
        }));
    }

    #[test]
    fn default_schema_rejects_oversized_payload() {
        let seed = CaseSeed {
            id: 1,
            payload: vec![0; 65],
        };
        let result = seed.validate(&SeedSchema::default());
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&SeedValidationError::PayloadTooLong {
            actual: 65,
            maximum: 64
        }));
    }

    #[test]
    fn default_schema_accepts_max_sized_payload() {
        let seed = CaseSeed {
            id: 1,
            payload: vec![0; 64],
        };
        assert!(seed.validate(&SeedSchema::default()).is_ok());
    }

    #[test]
    fn custom_schema_accepts_empty_payload() {
        let schema = SeedSchema::new(0, 100, 0, u64::MAX);
        let seed = CaseSeed {
            id: 1,
            payload: vec![],
        };
        assert!(seed.validate(&schema).is_ok());
    }

    #[test]
    fn custom_schema_rejects_id_below_min() {
        let schema = SeedSchema::new(0, 64, 10, u64::MAX);
        let seed = CaseSeed {
            id: 5,
            payload: vec![1, 2],
        };
        let result = seed.validate(&schema);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&SeedValidationError::IdTooSmall {
            actual: 5,
            minimum: 10
        }));
    }

    #[test]
    fn custom_schema_rejects_id_above_max() {
        let schema = SeedSchema::new(0, 64, 0, 100);
        let seed = CaseSeed {
            id: 200,
            payload: vec![1, 2],
        };
        let result = seed.validate(&schema);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.contains(&SeedValidationError::IdTooLarge {
            actual: 200,
            maximum: 100
        }));
    }

    #[test]
    fn validate_seed_uses_default_schema() {
        let seed = CaseSeed {
            id: 1,
            payload: vec![1, 2, 3],
        };
        assert!(validate_seed(&seed).is_ok());
    }

    #[test]
    fn validate_seed_with_schema_accepts_custom() {
        let seed = CaseSeed {
            id: 1,
            payload: vec![1, 2],
        };
        let schema = SeedSchema::new(1, 128, 0, 50);
        assert!(validate_seed_with_schema(&seed, &schema).is_ok());
    }

    #[test]
    fn multiple_errors_collected() {
        let seed = CaseSeed {
            id: 200,
            payload: vec![0; 100],
        };
        let schema = SeedSchema::new(5, 50, 0, 100);
        let result = seed.validate(&schema);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn error_display_shows_clear_message() {
        let err = SeedValidationError::PayloadTooLong {
            actual: 100,
            maximum: 64,
        };
        assert_eq!(err.to_string(), "payload too long: 100 bytes, maximum 64");
    }

    #[test]
    fn seed_schema_builder_with_payload_bounds() {
        let schema = SeedSchema::with_payload_bounds(1, 32);
        assert_eq!(schema.min_payload_len, 1);
        assert_eq!(schema.max_payload_len, 32);
    }

    #[test]
    fn seed_schema_builder_with_id_bounds() {
        let schema = SeedSchema::with_id_bounds(100, 1000);
        assert_eq!(schema.min_id, 100);
        assert_eq!(schema.max_id, 1000);
    }
}
