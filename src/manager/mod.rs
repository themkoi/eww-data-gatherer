//! Test application to list all available outputs.

use std::{error::Error};

use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};
use wayland_client::{globals::registry_queue_init, protocol::wl_output, Connection, QueueHandle};

use crate::config;

pub fn run() -> Result<(), Box<dyn Error>> {
    // We initialize the logger for the purpose of debugging.
    // Set `RUST_LOG=debug` to see extra debug information.

    // Try to connect to the Wayland server.
    let conn = Connection::connect_to_env()?;

    // Now create an event queue and a handle to the queue so we can create objects.
    let (globals, mut event_queue) = registry_queue_init(&conn).unwrap();
    let qh = event_queue.handle();

    // Initialize the registry handling so other parts of Smithay's client toolkit may bind
    // globals.
    let registry_state = RegistryState::new(&globals);

    // Initialize the delegate we will use for outputs.
    let output_delegate = OutputState::new(&globals, &qh);

    // Set up application state.
    //
    // This is where you will store your delegates and any data you wish to access/mutate while the
    // application is running.
    let mut list_outputs = ListOutputs {
        registry_state,
        output_state: output_delegate,
        initialized: false,
    };
    event_queue.blocking_dispatch(&mut list_outputs)?;
    // mark initialization complete
    list_outputs.initialized = true;

    // `OutputState::new()` binds the output globals found in `registry_queue_init()`.
    //
    // After the globals are bound, we need to dispatch again so that events may be sent to the newly
    // created objects.
    loop {
        event_queue.blocking_dispatch(&mut list_outputs)?;
    }
}

/// Application data.
///
/// This type is where the delegates for some parts of the protocol and any application specific data will
/// live.
struct ListOutputs {
    registry_state: RegistryState,
    output_state: OutputState,
    initialized: bool,
}

impl OutputHandler for ListOutputs {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        if self.initialized {
            let cfg = config::get_config();
            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(&cfg.com_on_output)
                .status()
                .expect("Failed to execute command");
            log::debug!("Command exited with: {}", status);
        }
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        let cfg = config::get_config();
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cfg.com_on_output)
            .status()
            .expect("Failed to execute command");
        log::debug!("Command exited with: {}", status);
    }
}

// Now we need to say we are delegating the responsibility of output related events for our application data
// type to the requisite delegate.
delegate_output!(ListOutputs);

// In order for our delegate to know of the existence of globals, we need to implement registry
// handling for the program. This trait will forward events to the RegistryHandler trait
// implementations.
delegate_registry!(ListOutputs);

// In order for delegate_registry to work, our application data type needs to provide a way for the
// implementation to access the registry state.
//
// We also need to indicate which delegates will get told about globals being created. We specify
// the types of the delegates inside the array.
impl ProvidesRegistryState for ListOutputs {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers! {
        // Here we specify that OutputState needs to receive events regarding the creation and destruction of
        // globals.
        OutputState,
    }
}
