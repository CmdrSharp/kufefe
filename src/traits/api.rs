use std::fmt::Debug;

use kube::Api;
use serde::de::DeserializeOwned;

pub trait ApiResource {
    type ApiType: Clone + DeserializeOwned + Debug;

    fn get_api(&self) -> Api<Self::ApiType>;
}
