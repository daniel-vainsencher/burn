use super::{LearnerCallback, LearnerItem};
use std::sync::{mpsc, Mutex};

enum Message<T, V> {
    LogTrain(LearnerItem<T>),
    LogValid(LearnerItem<V>),
    ClearTrain,
    ClearValid,
}

pub struct AsyncTrainerCallback<T, V> {
    sender: mpsc::Sender<Message<T, V>>,
}

#[derive(new)]
struct CallbackThread<T, V> {
    callback: Mutex<Box<dyn LearnerCallback<T, V>>>,
    receiver: mpsc::Receiver<Message<T, V>>,
}

impl<T, V> CallbackThread<T, V> {
    fn run(self) {
        for item in self.receiver.iter() {
            match item {
                Message::LogTrain(item) => {
                    let mut callback = self.callback.lock().unwrap();
                    callback.on_train_item(item);
                }
                Message::ClearTrain => {
                    let mut callback = self.callback.lock().unwrap();
                    callback.on_train_end_epoch();
                }
                Message::LogValid(item) => {
                    let mut callback = self.callback.lock().unwrap();
                    callback.on_valid_item(item);
                }
                Message::ClearValid => {
                    let mut callback = self.callback.lock().unwrap();
                    callback.on_valid_end_epoch();
                }
            }
        }
    }
}

impl<T: Send + Sync + 'static, V: Send + Sync + 'static> AsyncTrainerCallback<T, V> {
    pub fn new(callback: Box<dyn LearnerCallback<T, V>>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let thread = CallbackThread::new(Mutex::new(callback), receiver);

        std::thread::spawn(move || thread.run());

        Self { sender }
    }
}

impl<T: Send, V: Send> LearnerCallback<T, V> for AsyncTrainerCallback<T, V> {
    fn on_train_item(&mut self, item: LearnerItem<T>) {
        self.sender.send(Message::LogTrain(item)).unwrap();
    }

    fn on_valid_item(&mut self, item: LearnerItem<V>) {
        self.sender.send(Message::LogValid(item)).unwrap();
    }

    fn on_train_end_epoch(&mut self) {
        self.sender.send(Message::ClearTrain).unwrap();
    }

    fn on_valid_end_epoch(&mut self) {
        self.sender.send(Message::ClearValid).unwrap();
    }
}