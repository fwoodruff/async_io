use std::sync::mpsc;
#[allow(dead_code)]
pub struct MpscReceiver<T> {
    dd : mpsc::Receiver<T>,
}

#[allow(dead_code)]
pub struct MpscSender<T> {
    dd : mpsc::Sender<T>,
}