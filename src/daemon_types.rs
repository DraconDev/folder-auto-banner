use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::fs::DirSummary;
use crate::git::GitInfo;

// IPC Protocol
#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Banner { path: PathBuf },
    Warm { path: PathBuf },
    Ping,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Banner(Box<BannerData>),
    Pong,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BannerData {
    pub summary: DirSummary,
    pub git_info: Option<GitInfo>,
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn test_request_banner_serialization() {
        let request = Request::Banner {
            path: PathBuf::from("/tmp"),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Banner"));
        assert!(json.contains("/tmp"));

        let deserialized: Request = serde_json::from_str(&json).unwrap();
        match deserialized {
            Request::Banner { path } => assert_eq!(path, PathBuf::from("/tmp")),
            _ => panic!("Expected Banner variant"),
        }
    }

    #[test]
    fn test_request_warm_serialization() {
        let request = Request::Warm {
            path: PathBuf::from("/home"),
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Warm"));

        let deserialized: Request = serde_json::from_str(&json).unwrap();
        match deserialized {
            Request::Warm { path } => assert_eq!(path, PathBuf::from("/home")),
            _ => panic!("Expected Warm variant"),
        }
    }

    #[test]
    fn test_request_ping_serialization() {
        let request = Request::Ping;
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Ping"));

        let deserialized: Request = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Request::Ping));
    }

    #[test]
    fn test_request_shutdown_serialization() {
        let request = Request::Shutdown;
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Shutdown"));

        let deserialized: Request = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Request::Shutdown));
    }

    #[test]
    fn test_response_pong_serialization() {
        let response = Response::Pong;
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Pong"));

        let deserialized: Response = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, Response::Pong));
    }

    #[test]
    fn test_response_error_serialization() {
        let response = Response::Error {
            message: "test error".to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Error"));
        assert!(json.contains("test error"));

        let deserialized: Response = serde_json::from_str(&json).unwrap();
        match deserialized {
            Response::Error { message } => assert_eq!(message, "test error"),
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_response_dirsize_serialization() {
        let response = Response::DirSize {
            path: PathBuf::from("/tmp"),
            size: 12345,
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("DirSize"));
        assert!(json.contains("12345"));

        let deserialized: Response = serde_json::from_str(&json).unwrap();
        match deserialized {
            Response::DirSize { path, size } => {
                assert_eq!(path, PathBuf::from("/tmp"));
                assert_eq!(size, 12345);
            }
            _ => panic!("Expected DirSize variant"),
        }
    }

    #[test]
    fn test_banner_data_creation() {
        let data = crate::daemon_types::BannerData {
            summary: crate::fs::DirSummary::scan(Path::new("/tmp")).unwrap(),
            git_info: None,
        };

        assert!(data.git_info.is_none());
    }

    #[test]
    fn test_banner_data_serialization() {
        let data = crate::daemon_types::BannerData {
            summary: crate::fs::DirSummary::scan(Path::new("/tmp")).unwrap(),
            git_info: None,
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: crate::daemon_types::BannerData = serde_json::from_str(&json).unwrap();
        assert!(deserialized.git_info.is_none());
    }
}
