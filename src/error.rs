use crate::meta::{ProtocolStatus, RequestType};

use error_chain::error_chain;

error_chain! {
    foreign_links {
        Io(std::io::Error) #[doc = "Wapper of `std::io::Error`"];
    }

    errors {
        #[doc = "Usually not happen."]
        RequestIdNotFound(id: u16) {
            description("Request id not found."),
            display("Request id `{}` not found.", id),
        }

        #[doc = "Usually not happen."]
        ResponseNotFound(id: u16) {
            description("Response not found of request id."),
            display("Response not found of request id `{}`.", id),
        }

        #[doc = "Maybe unimplemented request type received fom response."]
        UnknownRequestType(r#type: RequestType) {
            description("Unknown request type."),
            display("Response not found of request id `{}`.", r#type),
        }

        #[doc = "Response not complete, first is protocol status and second is app status, see fastcgi protocol."]
        EndRequestCantMpxConn(app_status: u32) {
            description("End request error."),
            display("This app can't multiplex [CantMpxConn]; AppStatus: {}", app_status),
        }

        #[doc = "Response not complete, first is protocol status and second is app status, see fastcgi protocol."]
        EndRequestOverloaded(app_status: u32) {
            description("End request error."),
            display("New request rejected; too busy [OVERLOADED]; AppStatus: {}", app_status),
        }

        #[doc = "Response not complete, first is protocol status and second is app status, see fastcgi protocol."]
        EndRequestUnknownRole(app_status: u32) {
            description("End request error."),
            display("Role value not known [UnknownRole]; AppStatus: {}", app_status),
        }
    }
}

impl ErrorKind {
    pub(crate) fn new_end_request_with_protocol_status(
        protocol_status: ProtocolStatus,
        app_status: u32,
    ) -> Self {
        match protocol_status {
            ProtocolStatus::CantMpxConn => ErrorKind::EndRequestCantMpxConn(app_status),
            ProtocolStatus::Overloaded => ErrorKind::EndRequestOverloaded(app_status),
            _ => ErrorKind::EndRequestUnknownRole(app_status),
        }
    }
}
