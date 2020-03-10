use crate::{
    ClientState,
    Packet,
    RequestBuffer,
};

///A handler trait to customize the behavior of the cometd client
pub trait Handler {
    ///Fired on each recieved packet.
    ///# Return
    ///Return true to cancel processing. This will mess up dispatch to other handler functions.
    fn on_packet(&mut self, _p: &Packet) -> bool {
        false
    }

    ///Fired when a handshake packet is recieved
    fn on_handshake(&mut self, _client_state: &ClientState, _request_buffer: &mut RequestBuffer) {}

    ///Fired EACH time the client recieves a connect packet. Use a var if you only want to do an action on the first packet
    fn on_connect(&mut self, _client_state: &ClientState, _request_buffer: &mut RequestBuffer) {}

    ///Fired when an unknown packet is recieved. This is usually an event packet, so most processing will occur here.
    fn on_unknown(
        &mut self,
        _p: &Packet,
        _client_state: &ClientState,
        _request_buffer: &mut RequestBuffer,
    ) {
    }
}

/// A handler that does nothing.
pub struct NoopHandler;

impl Handler for NoopHandler {}
