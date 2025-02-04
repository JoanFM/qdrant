use crossbeam_channel::SendError;
use futures::io;
use schemars::JsonSchema;
use serde;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use serde_json::Error as JsonError;
use tokio::task::JoinError;
use std::result;

use segment::entry::entry_point::OperationError;
use segment::types::{Filter, PayloadKeyType, PayloadType, PointIdType, SearchParams, SeqNumberType, TheMap, VectorElementType};

use crate::config::CollectionConfig;
use crate::wal::WalError;

/// Type of vector in API
pub type VectorType = Vec<VectorElementType>;

#[derive(Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CollectionStatus {
    /// Collection if completely ready for requests
    Green,
    /// Collection is available, but some segments might be under optimization
    Yellow,
    /// Something is not OK
    Red
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Point data
pub struct Record {
    /// Id of the point
    pub id: PointIdType,
    /// Payload - values assigned to the point
    pub payload: Option<TheMap<PayloadKeyType, PayloadType>>,
    /// Vector of the point
    pub vector: Option<Vec<VectorElementType>>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
/// Current statistics and configuration of the collection.
pub struct CollectionInfo {
    /// Status of the collection
    pub status: CollectionStatus,
    /// Number of vectors in collection
    pub vectors_count: usize,
    /// Number of segments in collection
    pub segments_count: usize,
    /// Disk space, used by collection
    pub disk_data_size: usize,
    /// RAM used by collection
    pub ram_data_size: usize,
    /// Collection settings
    pub config: CollectionConfig,
}


#[derive(Debug, Deserialize, Serialize, JsonSchema, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStatus {
    /// Request is saved to WAL and will be process in a queue
    Acknowledged,
    /// Request is completed, changes are actual
    Completed,
}


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateResult {
    /// Sequential number of the operation
    pub operation_id: SeqNumberType,
    /// Update status
    pub status: UpdateStatus,
}


#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Search request
pub struct SearchRequest {
    /// Look for vectors closest to this
    pub vector: Vec<VectorElementType>,
    /// Look only for points which satisfies this conditions
    pub filter: Option<Filter>,
    /// Additional search params
    pub params: Option<SearchParams>,
    /// Max number of result to return
    pub top: usize,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Search request
pub struct RecommendRequest {
    /// Look for vectors closest to those
    pub positive: Vec<PointIdType>,
    /// Try to avoid vectors like this
    pub negative: Vec<PointIdType>,
    /// Look only for points which satisfies this conditions
    pub filter: Option<Filter>,
    /// Additional search params
    pub params: Option<SearchParams>,
    /// Max number of result to return
    pub top: usize,
}


#[derive(Error, Debug, Clone)]
#[error("{0}")]
pub enum CollectionError {
    #[error("Wrong input: {description}")]
    BadInput { description: String },
    #[error("No point with id {missed_point_id} found")]
    NotFound { missed_point_id: PointIdType },
    #[error("Service internal error: {error}")]
    ServiceError { error: String },
    #[error("Bad request: {description}")]
    BadRequest { description: String },
}

impl From<OperationError> for CollectionError {
    fn from(err: OperationError) -> Self {
        match err {
            OperationError::WrongVector { .. } => Self::BadInput { description: format!("{}", err) },
            OperationError::PointIdError { missed_point_id } => Self::NotFound { missed_point_id },
            OperationError::ServiceError { description } => Self::ServiceError { error: description },
            OperationError::TypeError { .. } => Self::BadInput { description: format!("{}", err) },
        }
    }
}

impl From<JoinError> for CollectionError {
    fn from(err: JoinError) -> Self {
        Self::ServiceError { error: format!("{}", err) }
    }
}

impl From<WalError> for CollectionError {
    fn from(err: WalError) -> Self {
        Self::ServiceError { error: format!("{}", err) }
    }
}

impl<T> From<SendError<T>> for CollectionError {
    fn from(_err: SendError<T>) -> Self {
        Self::ServiceError { error: format!("Can't reach one of the workers") }
    }
}

impl From<JsonError> for CollectionError {
    fn from(err: JsonError) -> Self {
        CollectionError::ServiceError { error: format!("Json error: {}", err) }
    }
}

impl From<io::Error> for CollectionError {
    fn from(err: io::Error) -> Self {
        CollectionError::ServiceError { error: format!("File IO error: {}", err) }
    }
}

pub type CollectionResult<T> = result::Result<T, CollectionError>;


