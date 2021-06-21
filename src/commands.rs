use super::{Error, Result, Device};

use reqwest::{Client, Response};
use serde_json::Value;


#[derive(Clone, Copy)]
// pub enum Command {
//     PairBegin,
//     PairFinish,
//     PairCancel,
// }

pub enum Request {
    Get,
    Put,
}

pub struct Command {
    endpoint: String,
    request: Request,
    value: Value,
}

// impl Device {
//     pub(crate) fn get(command: Command) ->
// }
