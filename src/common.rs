use std::{any::{Any, TypeId}, collections::BTreeMap, fmt}; // For custom error
#[derive(Debug)]
pub struct RsServiceError(
    pub String
);
impl std::error::Error for RsServiceError {

}
impl fmt::Display for RsServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RsService Error: {}", self.0)
    }
}

pub type ContainerStruct = Box<dyn Any>;
pub type MapForContainer = BTreeMap<TypeId, ContainerStruct>;