use std::sync::Arc;
use crate::implementation::futures::mpscimpl::ReceiverImpl;

pub struct AsyncReceiver<T> {
    pub(crate) shared_receiver : Arc<ReceiverImpl<T>>,
}
