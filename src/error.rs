use std::fmt;

use serde::Serialize;

#[derive(Debug, Copy, Clone, PartialEq, Serialize)]
pub enum ErrorKind {
    StorageCreateOrganizationFailure,
    StorageCreateOrganizationTimedOut,
    StorageGetAdminRoleIdFailure,
    LogicCreateOrganizationFailure,
    LogicCreateOrganizationTimedOut,
    StorageCreateMemberFailure,
}

#[derive(Debug, Clone, Serialize)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Error {
        Error {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl Into<cp_microservice::core::error::ErrorKind> for ErrorKind {
    fn into(self) -> cp_microservice::core::error::ErrorKind {
        match self {
            Self::LogicCreateOrganizationFailure | Self::LogicCreateOrganizationTimedOut => {
                cp_microservice::core::error::ErrorKind::LogicError
            }
            Self::StorageCreateOrganizationFailure
            | Self::StorageCreateOrganizationTimedOut
            | Self::StorageGetAdminRoleIdFailure
            | Self::StorageCreateMemberFailure => {
                cp_microservice::core::error::ErrorKind::StorageError
            }
            _ => cp_microservice::core::error::ErrorKind::Unknown,
        }
    }
}

impl Into<cp_microservice::core::error::Error> for Error {
    fn into(self) -> cp_microservice::core::error::Error {
        let error_kind: cp_microservice::core::error::ErrorKind = self.kind.into();

        cp_microservice::core::error::Error::new(error_kind, self.message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
