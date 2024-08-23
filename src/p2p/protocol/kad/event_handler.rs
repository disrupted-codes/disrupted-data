pub(crate) enum Event {
    Put(Put),
    Get(Get)
}

pub(crate) enum EventHandlerOutcome {
    Ok,
    Failed(String)
}


pub(crate) struct Put {
    key: Vec<u8>,
    value: Vec<u8>
}

impl Put {
    pub(crate) fn new(key: Vec<u8>, value: Vec<u8>) -> Self {
        Self {
            key,
            value
        }
    }

    pub(crate) fn handle(&self) -> EventHandlerOutcome {

        EventHandlerOutcome::Ok
    }
}
pub(crate) struct Get {
    value: Option<Vec<u8>>,
    error: Option<String>
}