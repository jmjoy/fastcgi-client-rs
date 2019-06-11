#[derive(Default, Debug)]
pub struct Params<'a> {
    pub gateway_interface: &'a str,
    pub request_method: &'a str,
    pub script_filename: &'a str,
    pub script_name: &'a str,
    pub query_string: &'a str,
    pub request_uri: &'a str,
    pub document_uri: &'a str,
    pub server_software: &'a str,
    pub remote_addr: &'a str,
    pub remote_port: &'a str,
    pub server_addr: &'a str,
    pub server_port: &'a str,
    pub server_name: &'a str,
    pub server_protocol: &'a str,
    pub content_type: &'a str,
    pub content_length: &'a str,
}

impl<'a> Params<'a> {
    pub fn with(
        request_method: &'a str,
        script_name: &'a str,
        query_string: &'a str,
        request_uri: &'a str,
        document_uri: &'a str,

        remote_addr: &'a str,
        remote_port: &'a str,
        server_addr: &'a str,
        server_port: &'a str,
        server_name: &'a str,
        content_type: &'a str,
        content_length: &'a str,
    ) -> Self {
        let mut params: Params = Default::default();

        params.request_method = request_method;
        params.script_name = script_name;
        params.query_string = query_string;
        params.request_uri = request_uri;
        params.document_uri = document_uri;
        params.remote_addr = remote_addr;
        params.remote_port = remote_port;
        params.server_addr = server_addr;
        params.server_port = server_port;
        params.server_name = server_name;
        params.content_type = content_type;
        params.content_length = content_length;

        params.gateway_interface = "FastCGI/1.0";
        params.server_software = "rust/fastcgi-client";
        params.server_protocol = "HTTP/1.1";

        params
    }
}

impl<'a> Into<HashMap<&'a str, &'a str>> for Params<'a> {
    fn into(self) -> HashMap<&'a str, &'a str> {
        let mut map = HashMap::new();
        map.insert("GATEWAY_INTERFACE", self.gateway_interface);
        map.insert("REQUEST_METHOD", self.request_method);
        map.insert("SCRIPT_FILENAME", self.script_name);
        map.insert("SCRIPT_NAME", self.script_name);
        map.insert("QUERY_STRING", self.query_string);
        map.insert("REQUEST_URI", self.request_uri);
        map.insert("DOCUMENT_URI", self.document_uri);
        map.insert("SERVER_SOFTWARE", self.server_software);
        map.insert("REMOTE_ADDR", self.remote_addr);
        map.insert("REMOTE_PORT", self.remote_port);
        map.insert("SERVER_ADDR", self.server_addr);
        map.insert("SERVER_PORT", self.server_port);
        map.insert("SERVER_NAME", self.server_name);
        map.insert("SERVER_PROTOCOL", self.server_protocol);
        map.insert("CONTENT_TYPE", self.content_type);
        map.insert("CONTENT_LENGTH", self.content_length);
        map
    }
}
