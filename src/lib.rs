use error::BoxError;
use log::debug;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::join;
use tokio::sync::mpsc::Sender;

#[macro_use]
mod error;
mod messages;

use error::Error;
use error::Result;
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
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
    doc_model: String,
    connection_request_tx: Sender<u32>,
    connection_request_rx: Receiver<u32>,
}

#[must_use]
pub struct ClientBuilder {
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

async fn serve_connection_request(
    req: Request<Body>,
    response_channel_tx: Sender<u32>,
) -> core::result::Result<Response<Body>, BoxError> {
    let response = if response_channel_tx.send(0).await.is_err() {
        Response::builder()
            .status(500)
            .header(CONTENT_TYPE, "text/html")
            .body(Body::from(""))
            .unwrap()
    } else {
        Response::builder()
            .status(200)
            .header(CONTENT_TYPE, "text/html")
            .body(Body::from(""))
            .unwrap()
    };
    Ok(response)
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
        if let Some(err) = self.config.error {
            return Err(err);
        }
        let (connection_request_tx, connection_request_rx) = mpsc::channel::<u32>(100);

        // create the hyper task
        let port = self.config.conn_req_port;
        let addr = ([127, 0, 0, 1], port).into();
        debug!("Listening for Connection Requests on http://{}", addr);
        let make_service = make_service_fn(move |_conn| {
            let cr_tx = connection_request_tx.clone();

            async move {
                Ok::<_, Infallible>(service_fn(move |req: Request<Body>| {
                    let cr_tx = cr_tx.clone();

                    serve_connection_request(req, cr_tx)
                }))
            }
        });

        Ok(Client {
            inner: Arc::new(ClientRef {
                http_client: reqwest::Client::new(),
                doc_model: self.config.doc_model.clone(),
                hyper_task: Server::bind(&addr).serve(make_service),
                connection_request_rx: connection_request_rx,
                connection_request_tx: connection_request_tx,
            }),
        })
    }
}

impl Client {
    /// Constructs a new `Client`.
    ///
    /// # Panics
    ///
    /// This method panics if a TLS backend cannot be initialized, or the resolver
    /// cannot load the system configuration.
    ///
    /// Use `Client::builder()` if you wish to handle the failure as an `Error`
    /// instead of panicking.
    pub fn new() -> Client {
        ClientBuilder::new().build().expect("Client::new()")
    }

    /// Creates a `ClientBuilder` to configure a `Client`.
    ///
    /// This is the same as `ClientBuilder::new()`.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// process all the message types.
    async fn process_messages(
        &self,
        mut periodic_rx: Receiver<u32>,
        mut notification_change_rx: Receiver<String>,
    ) {
        tokio::select! {
            periodic_timeout = periodic_rx.recv() => {
                dbg!(periodic_timeout);
            }
            connection_request = self.connection_request_rx.recv() => {
                dbg!(connection_request);
            }
            notification_param_changed = notification_change_rx.recv() => {
                dbg!(notification_param_changed);
            }
        }
    }

    async fn watch_notifications(&self, notification_change_tx: Sender<String>) {}

    /// Starts the tasks.
    ///
    /// A connection request listener task
    /// The timers
    /// Document model change listener (TODO: How to discover these types of events?)
    /// The message processor
    ///
    /// Once the tasks are started, send the initial INFORM
    async fn run(&self) {
        let (periodic_tx, periodic_rx) = mpsc::channel::<u32>(100);
        let (notification_change_tx, notification_change_rx) = mpsc::channel::<String>(100);

        // if periodic informs are enabled, start a timer to handle it

        let message_processor = self.process_messages(periodic_rx, notification_change_rx);
        let notification_watcher = self.watch_notifications(notification_change_tx);
        let res = join!(message_processor, notification_watcher);
    }
}

#[cfg(test)]
mod tests {
    use super::Client;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn connection_requests() {
        let client = Client::new();
        assert!(true);
    }
}
