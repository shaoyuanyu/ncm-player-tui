use serde::Deserialize;

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct QrResponse<D> {
    pub code: usize,
    pub data: D,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct QrKeyData {
    pub code: usize,
    pub unikey: String,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct QrCreateData {
    pub qrurl: String,
    pub qrimg: String,
}

#[allow(unused)]
#[derive(Deserialize, Debug)]
pub struct QrCheckResponse {
    pub code: usize,
    pub message: String,
    pub cookie: String,
}
