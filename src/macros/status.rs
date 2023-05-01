#[macro_export]
macro_rules! status_update {
        ($fn_name:ident, $( $field_name:ident : $field_ty:ty ),+) => {
        /// Updates fields on RequestStatus
        pub fn $fn_name(&mut self, $($field_name: $field_ty),+) -> &mut Self {
            if self.status.is_none() {
                self.status = Some(RequestStatus::default());
            }

            self.status = Some(RequestStatus {
                $($field_name,)+
                ..self.status.take().unwrap()
            });

            self
        }
    };
}
