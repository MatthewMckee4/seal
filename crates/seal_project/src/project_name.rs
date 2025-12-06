use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::error::ConfigValidationError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ProjectName(String);

impl ProjectName {
    pub fn new(value: String) -> Result<Self, ConfigValidationError> {
        if value.trim().is_empty() {
            return Err(ConfigValidationError::EmptyProjectName);
        }

        if !value
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ConfigValidationError::InvalidProjectName {
                name: value.clone(),
            });
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for ProjectName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for ProjectName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_project_names() {
        assert!(ProjectName::new("my-project".to_string()).is_ok());
        assert!(ProjectName::new("my_project".to_string()).is_ok());
        assert!(ProjectName::new("MyProject123".to_string()).is_ok());
        assert!(ProjectName::new("project-name_123".to_string()).is_ok());
    }

    #[test]
    fn test_empty_project_name() {
        let result = ProjectName::new(String::new());
        assert!(result.is_err());
        insta::assert_debug_snapshot!(result.unwrap_err(), @"EmptyProjectName");
    }

    #[test]
    fn test_whitespace_only_project_name() {
        let result = ProjectName::new("   ".to_string());
        assert!(result.is_err());
        insta::assert_debug_snapshot!(result.unwrap_err(), @"EmptyProjectName");
    }

    #[test]
    fn test_invalid_characters() {
        let result = ProjectName::new("my project".to_string());
        assert!(result.is_err());
        insta::assert_debug_snapshot!(result.unwrap_err(), @r#"
        InvalidProjectName {
            name: "my project",
        }
        "#);

        let result = ProjectName::new("my/project".to_string());
        assert!(result.is_err());

        let result = ProjectName::new("my@project".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_project_name_display() {
        let name = ProjectName::new("test-project".to_string()).unwrap();
        assert_eq!(name.to_string(), "test-project");
        assert_eq!(name.as_str(), "test-project");
    }

    #[test]
    fn test_project_name_ordering() {
        let name1 = ProjectName::new("aaa".to_string()).unwrap();
        let name2 = ProjectName::new("bbb".to_string()).unwrap();
        assert!(name1 < name2);
    }

    #[test]
    fn test_serialization() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Wrapper {
            name: ProjectName,
        }

        let name = ProjectName::new("my-project".to_string()).unwrap();

        let wrapper = Wrapper { name: name.clone() };
        let toml_string = toml::to_string(&wrapper).unwrap();
        assert!(toml_string.contains(r#"name = "my-project""#));

        let deserialized: Wrapper = toml::from_str(&toml_string).unwrap();
        assert_eq!(deserialized.name, name);
    }
}
