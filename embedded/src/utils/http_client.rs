use embedded_svc::{http::client::Client, utils::io};
use esp_idf_svc::http::client::EspHttpConnection;
use log::error;
use log::info;

pub struct HttpClient {
    client: Client<EspHttpConnection>,
}

impl HttpClient {
    pub fn new() -> anyhow::Result<Self> {
        let client = Client::wrap(EspHttpConnection::new(&Default::default())?);
        Ok(Self { client })
    }

    pub fn post<'a>(
        &mut self,
        uri: &'a str,
        headers: &'a [(&'a str, &'a str)],
        payload: &'a [u8],
    ) -> anyhow::Result<()> {
        let mut request = self.client.post(uri, headers)?;
        request.write(payload)?;

        let mut response = request.submit()?;
        let status = response.status();

        let mut buf = [0u8; 1024];
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0)?;
        match std::str::from_utf8(&buf[0..bytes_read]) {
            Ok(body_string) => info!("POST {} {} : {:?}", uri, status, body_string),
            Err(e) => error!(
                "POST {} {} : Error decoding response body: {}",
                uri, status, e
            ),
        };

        Ok(())
    }
}
