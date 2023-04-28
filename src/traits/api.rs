use kube::Api;

pub trait ApiResource {
    type ApiType;

    fn get_api(&self) -> Api<Self::ApiType>;
}
