use midir::{InitError, MidiIO, MidiInput, MidiOutput};

const CLIENT_NAME: &str = "hmrc";

fn print_ports<T: MidiIO>(label: &str, io: &T) {
    // ports() is a snapshot taken at the "moment of invocation."
    // Must call this again (connect or disconnect a cable)
    let ports = io.ports();
    println!("[{}] {} port(s)", label, ports.len());

    if ports.is_empty() {
        println!("IO (none) - check the USB cable and that the piano is powered on");
        return;
    }

    for (index, port) in ports.iter().enumerate() {
        match io.port_name(port) {
            Ok(name) => println!("{}: {}", index, name),
            Err(e) => print!("{}: (unavailable: {})", index, e),
        }
    }
}

fn main() -> Result<(), InitError> {
    let midi_in = MidiInput::new(CLIENT_NAME)?;
    let midi_out = MidiOutput::new(CLIENT_NAME)?;

    print_ports("Input", &midi_in);
    print_ports("Output", &midi_out);

    Ok(())
}
