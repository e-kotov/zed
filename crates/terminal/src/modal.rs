use gpui::{ModelHandle, ViewContext};
use workspace::Workspace;

use crate::{get_wd_for_workspace, DeployModal, Event, Terminal, TerminalConnection};

struct StoredConnection(ModelHandle<TerminalConnection>);

pub fn deploy_modal(workspace: &mut Workspace, _: &DeployModal, cx: &mut ViewContext<Workspace>) {
    // Pull the terminal connection out of the global if it has been stored
    let possible_connection =
        cx.update_default_global::<Option<StoredConnection>, _, _>(|possible_connection, _| {
            possible_connection.take()
        });

    if let Some(StoredConnection(stored_connection)) = possible_connection {
        // Create a view from the stored connection
        workspace.toggle_modal(cx, |_, cx| {
            cx.add_view(|cx| Terminal::from_connection(stored_connection, true, cx))
        });
    } else {
        // No connection was stored, create a new terminal
        if let Some(closed_terminal_handle) = workspace.toggle_modal(cx, |workspace, cx| {
            let wd = get_wd_for_workspace(workspace, cx);
            let this = cx.add_view(|cx| Terminal::new(wd, true, cx));
            let connection_handle = this.read(cx).connection.clone();
            cx.subscribe(&connection_handle, on_event).detach();
            this
        }) {
            let connection = closed_terminal_handle.read(cx).connection.clone();
            cx.set_global(Some(StoredConnection(connection)));
        }
    }
}

pub fn on_event(
    workspace: &mut Workspace,
    _: ModelHandle<TerminalConnection>,
    event: &Event,
    cx: &mut ViewContext<Workspace>,
) {
    // Dismiss the modal if the terminal quit
    if let Event::CloseTerminal = event {
        cx.set_global::<Option<StoredConnection>>(None);
        if workspace
            .modal()
            .cloned()
            .and_then(|modal| modal.downcast::<Terminal>())
            .is_some()
        {
            workspace.dismiss_modal(cx)
        }
    }
}
