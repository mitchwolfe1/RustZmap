use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr},
    thread,
    time::Duration,
};

use anyhow::Result;
use crossbeam_channel as chan;
use etherparse::PacketBuilder;
use ipnet::Ipv4Net;

use pnet::{
    packet::{
        ip::IpNextHeaderProtocols,
        Packet,
        ipv4::Ipv4Packet,
        tcp::TcpPacket,
    },
    transport::{
        transport_channel,
        TransportChannelType::Layer3,
        TransportReceiver,
        TransportSender,
        ipv4_packet_iter,
    },
};

fn send_tcp_syn(
    tx: &mut TransportSender,
    src_ip: Ipv4Addr,
    dst_ip: Ipv4Addr,
    src_port: u16,
    dst_port: u16,
) -> Result<()> {
    let builder = PacketBuilder::ipv4(src_ip.octets(), dst_ip.octets(), 64)
        .tcp(src_port, dst_port, 0, 64_240)
        .syn();

    let mut buf = vec![0u8; builder.size(0)];
    builder.write(&mut buf, &[])?; // empty payload
    tx.send_to(Ipv4Packet::new(&buf).unwrap(), IpAddr::V4(dst_ip))?;
    Ok(())
}

fn first_local_ipv4() -> Result<Ipv4Addr> {
    for iface in pnet::datalink::interfaces() {
        if !iface.is_up() || iface.is_loopback() {
            continue;
        }
        for ip in iface.ips {
            if let IpAddr::V4(v4) = ip.ip() {
                return Ok(v4);
            }
        }
    }
    anyhow::bail!("couldn’t find a non-loopback v4 address")
}

pub fn scan_subnet(subnet: &str, dst_port: u16, timeout: Duration) -> Result<()> {
    let net: Ipv4Net = subnet.parse()?;
    let src_ip = first_local_ipv4()?;
    let src_port = 50_000;

    let (mut tx, mut rx): (TransportSender, TransportReceiver) =
        transport_channel(4096, Layer3(IpNextHeaderProtocols::Tcp))?;

    let (found_tx, found_rx) = chan::unbounded::<Ipv4Addr>();

    let found_tx_for_thread = found_tx.clone();

    thread::spawn(move || {
        let mut iter = ipv4_packet_iter(&mut rx);
        while let Ok((ip_pkt, _)) = iter.next() {
            let ihl_bytes = (ip_pkt.get_header_length() as usize) * 4;
            let full_packet = ip_pkt.packet();
            if full_packet.len() <= ihl_bytes {
                continue;
            }
            let tcp_payload = &full_packet[ihl_bytes..];
            if let Some(tcp) = TcpPacket::new(tcp_payload) {
                let flags = tcp.get_flags();
                let syn_ack = flags & 0x12 == 0x12;
                if syn_ack && tcp.get_destination() == src_port {
                    found_tx_for_thread.send(ip_pkt.get_source()).ok();
                }
            }
        }
    });

    for dst_ip in net.hosts() {
        println!("scanning {dst_ip}");
        send_tcp_syn(&mut tx, src_ip, dst_ip, src_port, dst_port)?;
    }

    thread::sleep(timeout);

    drop(found_tx);

    let responders: HashSet<_> = found_rx.iter().collect();
    println!("Open on port {dst_port}:");
    for ip in responders {
        println!("  {ip}");
    }

    Ok(())
}


