use std::error::Error;
use std::io::{self, Write};

use midir::{Ignore, MidiIO, MidiInput, MidiInputPort};

const CLIENT_NAME: &str = "hmrc";

fn main() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new(CLIENT_NAME)?;

    midi_in.ignore(Ignore::None);

    let port = select_port(&midi_in)?;
    let port_name = midi_in.port_name(&port)?;
    let mut last_stamp: Option<u64> = None;

    let _conn = midi_in.connect(
        &port,
        "hmrc-input",
        move |stamp_us, bytes, _| {
            let delta_us = last_stamp.map_or(0, |prev| stamp_us.saturating_sub(prev));
            last_stamp = Some(stamp_us);

            println!(
                "{:>12} us  (+{:>8} us)  {}",
                stamp_us,
                delta_us,
                to_hex(bytes)
            );
        },
        (),
    )?;

    println!("Connected to '{port_name}'. Play some keys. Press Enter to quit.");
    wait_for_enter()?;

    println!("Disconnected.");
    Ok(())
}

fn select_port(midi_in: &MidiInput) -> Result<MidiInputPort, Box<dyn Error>> {
    let ports = midi_in.ports();

    match ports.len() {
        0 => Err("no MIDI input port found (is the piano connected and powered on?)".into()),
        1 => {
            println!("Only one input port found. Selecting it automatically.");
            Ok(ports[0].clone())
        }
        _ => {
            print_ports("Input", midi_in);
            print!("Select input port number: ");
            io::stdout().flush()?;

            let mut line = String::new();
            io::stdin().read_line(&mut line)?;
            let index: usize = line.trim().parse()?;

            ports
                .get(index)
                .cloned()
                .ok_or_else(|| format!("invalid port number: {index}").into())
        }
    }
}

fn print_ports<T: MidiIO>(label: &str, io: &T) {
    let ports = io.ports();

    println!("[{label}] {} port(s)", ports.len());

    for (index, port) in ports.iter().enumerate() {
        match io.port_name(port) {
            Ok(name) => println!("  {index}: {name}"),
            Err(e) => println!("  {index}: <unavailable: {e}>"),
        }
    }
}

/// [0x90, 0x3C, 0x64] -> "90 3C 64"
fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn wait_for_enter() -> io::Result<()> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(())
}