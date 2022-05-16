use std::fmt;
use std::sync::Arc;

#[macro_use]
mod error;
mod messages;

use error::Error;
use error::Result;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

/// The config struct is for client settings
///
/// like which document model to support.
/// which port to listen to connection requests on
/// aso.
///

/// An asynchronous `Client` to make Requests with.
///
/// The Client has various configuration values to tweak, but the defaults
/// are set to what is usually the most commonly desired value. To configure a
/// `Client`, use `Client::builder()`.
///
/// The `Client` holds a connection pool internally, so it is advised that
/// you create one and **reuse** it.
///
/// You do **not** have to wrap the `Client` in an [`Rc`] or [`Arc`] to **reuse** it,
/// because it already uses an [`Arc`] internally.
///
/// [`Rc`]: std::rc::Rc
#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientRef>,
}

struct ClientRef {
    // A way to transfer messages upstream
    http_client: reqwest::Client,
}

#[must_use]
struct ClientBuilder {
    config: Config,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct Config {
    error: Option<Error>,
    doc_model: String,
    conn_req_port: u16,
}

impl ClientBuilder {
    /// Constructs a new `ClientBuilder`.
    ///
    /// This is the same as `Client::builder()`.
    pub fn new() -> ClientBuilder {
        ClientBuilder {
            config: Config {
                error: None,
                doc_model: String::from("1.4"),
                conn_req_port: 7547,
            },
        }
    }

    /// Returns a `Client` that uses this `ClientBuilder` configuration.
    ///
    /// # Errors
    ///
    /// This method fails if a TLS backend cannot be initialized, or the resolver
    /// cannot load the system configuration.
    pub fn build(self) -> crate::Result<Client> {
        let config = self.config;

        if let Some(err) = config.error {
            return Err(err);
        }

        Ok(Client {
            inner: Arc::new(ClientRef {
                http_client: reqwest::Client::new(),
            }),
        })
    }

    /// process all the message types.
    async fn process_messages(
        mut periodic_rx: Receiver<u32>,
        mut connection_request_rx: Receiver<u32>,
        mut notification_change_rx: Receiver<String>,
    ) {
        tokio::select! {
            periodic_timeout = periodic_rx.recv() => {
                dbg!(periodic_timeout);
            }
            connection_request = connection_request_rx.recv() => {
                dbg!(connection_request);

            }
            notification_param_changed = notification_change_rx.recv() => {
                dbg!(notification_param_changed);
            }
        }
    }

    /// Starts the tasks.
    ///
    /// A connection request listener task
    /// The timers
    /// Document model change listener (TODO: How to discover these types of events?)
    /// The message processor
    ///
    /// Once the tasks are started, send the initial INFORM
    async fn run() {
        let (periodic_tx, periodic_rx) = mpsc::channel::<u32>(100);

        // if periodic informs are enabled, start a timer to handle it
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
