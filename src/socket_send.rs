struct SocketSend {
    socket: UdpSocket,
    buffer: [u8; 1500],
    adr: SocketAddr,
}

