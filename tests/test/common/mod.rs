// adapted from bevy_replicon

//local shortcuts
use bevy_replicon_attributes::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_renet::renet::{
    transport::{
        ClientAuthentication, NetcodeClientTransport, NetcodeServerTransport, ServerAuthentication,
        ServerConfig,
    },
    ClientId, ConnectionConfig, RenetClient, RenetServer,
};
use bevy_replicon::prelude::*;

//standard shortcuts
use std::{
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    time::SystemTime,
};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

const PROTOCOL_ID: u64 = 0;

fn create_server_transport() -> NetcodeServerTransport {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0);
    let socket = UdpSocket::bind(server_addr).expect("localhost should be bindable");
    let public_addr = socket
        .local_addr()
        .expect("socket should autodetect local address");
    let server_config = ServerConfig {
        current_time,
        max_clients: 10,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    NetcodeServerTransport::new(server_config, socket).unwrap()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn create_client_transport(client_id: u64, port: u16) -> NetcodeClientTransport {
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let ip = Ipv4Addr::LOCALHOST.into();
    let server_addr = SocketAddr::new(ip, port);
    let socket = UdpSocket::bind((ip, 0)).expect("localhost should be bindable");
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    NetcodeClientTransport::new(current_time, authentication, socket).unwrap()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub(super) fn connect(server_app: &mut App, client_app: &mut App) -> (u64, u16)
{
    let network_channels = server_app.world.resource::<NetworkChannels>();

    let server_channels_config = network_channels.get_server_configs();
    let client_channels_config = network_channels.get_client_configs();

    let server = RenetServer::new(ConnectionConfig {
        server_channels_config: server_channels_config.clone(),
        client_channels_config: client_channels_config.clone(),
        ..Default::default()
    });
    let client = RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let server_transport = create_server_transport();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let server_port = server_transport.addresses().first().unwrap().port();
    let client_transport = create_client_transport(client_id, server_port);

    server_app
        .insert_resource(server)
        .insert_resource(server_transport);

    client_app
        .insert_resource(client)
        .insert_resource(client_transport);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        client_app.update();
        server_app.update();
        if client_app.world.resource::<RenetClient>().is_connected() {
            break;
        }
    }

    (client_id, server_port)
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn reconnect(server_app: &mut App, client_app: &mut App, client_id: u64, server_port: u16)
{
    let network_channels = client_app.world.resource::<NetworkChannels>();
    let server_channels_config = network_channels.get_server_configs();
    let client_channels_config = network_channels.get_client_configs();

    let client = RenetClient::new(ConnectionConfig {
        server_channels_config,
        client_channels_config,
        ..Default::default()
    });

    let client_transport = create_client_transport(client_id, server_port);

    client_app
        .insert_resource(client)
        .insert_resource(client_transport);

    loop {
        std::thread::sleep(std::time::Duration::from_millis(10));
        client_app.update();
        server_app.update();
        if client_app.world.resource::<RenetClient>().is_connected() {
            break;
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn add_attribute<T: VisibilityAttribute>(In((id, attribute)): In<(u64, T)>, mut attributes: ClientAttributes)
{
    attributes.add(ClientId::from_raw(id), attribute);
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn remove_attribute<T: VisibilityAttribute>(In((id, attribute)): In<(u64, T)>, mut attributes: ClientAttributes)
{
    attributes.remove(ClientId::from_raw(id), attribute);
}

//-------------------------------------------------------------------------------------------------------------------
