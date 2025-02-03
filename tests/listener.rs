use ap_listener::{Command, Listener};
use ap_sc_notifier::SimulationControllerNotifier;
use ap_transmitter::LogicCommand;
use crossbeam_channel::{unbounded, Receiver, Sender};
use messages::node_event::NodeEvent;
use messages::Message;
use ntest::timeout;
use std::sync::Arc;
use std::thread;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{Ack, Fragment, Nack, NackType, Packet, PacketType};

fn create_simulation_controller_notifier(
) -> (Arc<SimulationControllerNotifier>, Receiver<NodeEvent>) {
    let (simulation_controller_tx, simulation_controller_rx) = unbounded();

    let simulation_controller_notifier =
        SimulationControllerNotifier::new(simulation_controller_tx);
    let simulation_controller_notifier = Arc::new(simulation_controller_notifier);

    (simulation_controller_notifier, simulation_controller_rx)
}

fn create_listener(
    node_id: NodeId,
    simulation_controller_notifier: Arc<SimulationControllerNotifier>,
) -> (
    Listener,
    Receiver<LogicCommand>,
    Receiver<Message>,
    Sender<Packet>,
    Sender<Command>,
) {
    let (listener_to_transmitter_tx, listener_to_transmitter_rx) = unbounded();
    let (listener_to_logic_tx, listener_to_logic_rx) = unbounded();
    let (drones_to_listener_tx, drones_to_listener_rx) = unbounded();
    let (listener_command_tx, listener_command_rx) = unbounded();

    let listener = Listener::new(
        node_id,
        listener_to_transmitter_tx,
        listener_to_logic_tx,
        drones_to_listener_rx,
        listener_command_rx,
        simulation_controller_notifier,
    );

    (
        listener,
        listener_to_transmitter_rx,
        listener_to_logic_rx,
        drones_to_listener_tx,
        listener_command_tx,
    )
}

#[test]
#[timeout(2000)]
fn check_quit_command() -> thread::Result<()> {
    let node_id = 0;

    let (simulation_controller_notifier, simulation_controller_rx) =
        create_simulation_controller_notifier();
    let (
        mut listener,
        listener_to_transmitter_rx,
        listener_to_logic_rx,
        drones_to_listener_tx,
        listener_command_tx,
    ) = create_listener(node_id, simulation_controller_notifier.clone());

    let handle = thread::spawn(move || {
        listener.run();
    });

    let _ = listener_command_tx.send(Command::Quit);

    handle.join()
}

/*
#[test]
#[timeout(2000)]
fn check_internal_transmitter_to_listener_channel() {
    let node_id = 0;

    let (simulation_controller_notifier, simulation_controller_rx) = create_simulation_controller_notifier();
    let (mut listener,
        listener_to_transmitter_rx,
        transmitter_to_listener_tx,
        listener_to_logic_rx,
        drones_to_listener_tx,
        listener_command_tx) = create_listener(node_id, simulation_controller_notifier.clone());

    let handle = thread::spawn(move || {
        listener.run();
    });

    let nack = Nack {
        fragment_index: 0,
        nack_type: NackType::ErrorInRouting(10),
    };
    let nack = Packet {
        routing_header: SourceRoutingHeader { hop_index: 0, hops: vec![node_id] },
        session_id: 0,
        pack_type: PacketType::Nack(nack),
    };

    transmitter_to_listener_tx.send(nack).expect("Transmitter cannot communicate with listener");

    let received = listener_to_transmitter_rx.recv().unwrap();

    assert_eq!(received, nack);
}
 */

#[test]
#[timeout(2000)]
fn check_unexpected_recipient() {
    let node_id = 0;

    let (simulation_controller_notifier, simulation_controller_rx) =
        create_simulation_controller_notifier();
    let (
        mut listener,
        listener_to_transmitter_rx,
        listener_to_logic_rx,
        drones_to_listener_tx,
        listener_command_tx,
    ) = create_listener(node_id, simulation_controller_notifier.clone());

    let handle = thread::spawn(move || {
        listener.run();
    });

    let fragment = Fragment {
        fragment_index: 0,
        total_n_fragments: 1,
        length: 128,
        data: [0; 128],
    };
    let packet = Packet {
        routing_header: SourceRoutingHeader {
            hop_index: 2,
            hops: vec![100, node_id, 1],
        },
        session_id: 0,
        pack_type: PacketType::MsgFragment(fragment),
    };

    drones_to_listener_tx
        .send(packet.clone())
        .expect("Transmitter cannot communicate with listener");

    let received = listener_to_transmitter_rx.recv().unwrap();

    let expected = LogicCommand::SendNack {
        session_id: 0,
        nack: Nack {
            fragment_index: 0,
            nack_type: NackType::UnexpectedRecipient(node_id),
        },
        destination: 100,
    };

    assert_eq!(received, expected);

    let fragment = Fragment {
        fragment_index: 0,
        total_n_fragments: 1,
        length: 128,
        data: [0; 128],
    };

    let packet = Packet {
        routing_header: SourceRoutingHeader {
            hop_index: 0,
            hops: vec![100, node_id],
        },
        session_id: 0,
        pack_type: PacketType::MsgFragment(fragment),
    };

    drones_to_listener_tx
        .send(packet.clone())
        .expect("Transmitter cannot communicate with listener");

    let received = listener_to_transmitter_rx.recv().unwrap();

    let expected = LogicCommand::SendNack {
        session_id: 0,
        nack: Nack {
            fragment_index: 0,
            nack_type: NackType::UnexpectedRecipient(node_id),
        },
        destination: 100,
    };

    assert_eq!(received, expected);
}
