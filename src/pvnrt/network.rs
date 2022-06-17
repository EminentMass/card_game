struct Network {
    containers: Vec<Container>,
    connections: Vec<Connection>,
}

enum ConnectionEndpoint {
    Void,
    Blocked,
    Connection(u32),
}

struct Connection {
    start: ConnectionEndpoint,
    end: ConnectionEndpoint,
}

struct CylinderContainer {
    length: f32,
    radius: f32,
}

struct JunctionContainer {
    radius: f32,
    connections: u32,
}

enum Container {}
