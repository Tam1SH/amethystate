use rpstate::ReactiveMap;
use rpstate_macros::{rpstate, RpType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, RpType)]
pub struct AlertThresholds {
    pub warning: u64,
    pub critical: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, RpType)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub thresholds: AlertThresholds,
}

#[rpstate]
pub struct DatabaseConfig {
    #[state(default = "localhost".to_string())]
    pub host: String,
}

#[rpstate(prefix = "sys")]
pub struct SystemSettings {

    #[state(nested)]
    pub db: DatabaseConfig,

    #[state(default = MonitoringConfig {
        enabled: true,
        thresholds: AlertThresholds { warning: 50, critical: 80 }
    })]
    pub monitoring: MonitoringConfig,

    #[state(default = {
        "cpu": AlertThresholds { warning: 70, critical: 90 },
        "mem": AlertThresholds { warning: 80, critical: 95 }
    })]
    pub limits: ReactiveMap<String, AlertThresholds>,

    #[state(default = [
        AlertThresholds { warning: 10, critical: 20 },
        AlertThresholds { warning: 30, critical: 40 }
    ])]
    pub presets: Vec<AlertThresholds>,
}

fn main() {}