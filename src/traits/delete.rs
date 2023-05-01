use async_trait::async_trait;
use either::Either;
use kube::{api::DeleteParams, core::Status, error::ErrorResponse, Api, Error};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

#[async_trait]
pub trait DeleteOpt<T> {
    async fn delete_opt(
        &self,
        name: &str,
        dp: &DeleteParams,
    ) -> Result<Option<Either<T, Status>>, Error>;
}

#[async_trait]
impl<T> DeleteOpt<T> for Api<T>
where
    T: Clone + DeserializeOwned + Debug,
{
    async fn delete_opt(
        &self,
        name: &str,
        dp: &DeleteParams,
    ) -> Result<Option<Either<T, Status>>, Error> {
        match self.delete(name, dp).await {
            Ok(obj) => Ok(Some(obj)),
            Err(Error::Api(ErrorResponse { code: 404, .. })) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
