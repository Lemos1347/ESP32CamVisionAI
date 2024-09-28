pub struct MultiPartForm<'a> {
    form: Vec<u8>,
    boundary: &'a str,
    pub content_type: &'a str,
}

impl<'a> MultiPartForm<'a> {
    pub fn new() -> Self {
        MultiPartForm {
            form: Vec::new(),
            boundary: "----WebKitFormBoundary7MA4YWxkTrZu0gW",
            content_type: "multipart/form-data; boundary=----WebKitFormBoundary7MA4YWxkTrZu0gW",
        }
    }

    pub fn add_file(&mut self, field: &str, buf: &[u8]) {
        let start_content: String = format!("--{}\r\n", self.boundary);
        let form_content = format!(
            "Content-Disposition: form-data; name=\"{}\"; filename=\"teste.jpg\"\r\nContent-Type: image/jpeg\r\n\r\n",
            field
        );

        self.form.extend_from_slice(&start_content.into_bytes());
        self.form.extend_from_slice(&form_content.into_bytes());
        self.form.extend_from_slice(buf);
    }

    pub fn wrap_up(&mut self) -> Vec<u8> {
        let close_form = format!("\r\n--{}--\r\n", self.boundary);

        self.form.extend_from_slice(&close_form.into_bytes());

        std::mem::take(&mut self.form)
    }
}
