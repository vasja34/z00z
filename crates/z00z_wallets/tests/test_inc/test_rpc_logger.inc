use std::sync::{Arc, OnceLock};

use z00z_utils::logger::{Logger, VecLogger};

use z00z_wallets::rpc::logging::{build_rpc_file_logger, RpcLoggingConfig};

struct TeeLogger {
    a: Arc<dyn Logger>,
    b: Arc<dyn Logger>,
}

impl TeeLogger {
    fn new(a: Arc<dyn Logger>, b: Arc<dyn Logger>) -> Self {
        Self { a, b }
    }
}

impl Logger for TeeLogger {
    fn error(&self, message: &str) {
        self.a.error(message);
        self.b.error(message);
    }

    fn warn(&self, message: &str) {
        self.a.warn(message);
        self.b.warn(message);
    }

    fn info(&self, message: &str) {
        self.a.info(message);
        self.b.info(message);
    }

    fn debug(&self, message: &str) {
        self.a.debug(message);
        self.b.debug(message);
    }

    fn trace(&self, message: &str) {
        self.a.trace(message);
        self.b.trace(message);
    }
}

fn ensure_test_log_origin() {
    static SET_ORIGIN: OnceLock<()> = OnceLock::new();
    SET_ORIGIN.get_or_init(|| {
        std::env::set_var("Z00Z_RPC_LOG_ORIGIN", "test");
    });
}

pub fn rpc_test_tee_logger() -> (Arc<dyn Logger>, Arc<VecLogger>) {
    ensure_test_log_origin();

    let vec_logger = Arc::new(VecLogger::new());
    let file_logger = build_rpc_file_logger(
        &RpcLoggingConfig::from_default_wallet_yaml().expect("RPC logging config must load"),
    )
    .expect("RPC log sink must initialize");
    let logger: Arc<dyn Logger> = Arc::new(TeeLogger::new(file_logger, vec_logger.clone()));
    (logger, vec_logger)
}
