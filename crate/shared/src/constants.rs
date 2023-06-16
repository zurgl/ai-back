pub const AI_DATA_PATH: &str = ".ai-data";

pub mod role {
    pub const ROOT: &str = "ROOT";
}

pub mod chan {
    pub const MPSC_LEN: usize = 200;
    pub const BORDCAST_LEN: usize = 200;
}

pub mod time {
    pub const VOLATILE: u64 = 25;
    pub const FOREVER: u64 = u64::MAX;
    pub const INTERVAL: u64 = 2_000;
}

pub mod route {
    pub const HEALTH_URL: &str = "/health";
    pub const SSE_URL: &str = "/sse";
    pub const API_COMMAND_PROCESS_URL: &str = "/command/process";
    pub const API_COMMAND_KILL_URL: &str = "/command/kill";
    pub const API_COMMAND_PAUSE_URL: &str = "/command/pause";
    pub const API_COMMAND_RESUME_URL: &str = "/command/resume";
    pub const API_COMMAND_SPAWN_URL: &str = "/command/spawn";
    pub const ROOT_URL: &str = "/";
}

pub mod server {
    pub const PORT: u16 = 7443;
    pub const ADDRESS: [u8; 4] = [0, 0, 0, 0];
    pub const CORS_ALLOWED_ORIGINS: [&str; 10] = [
        "https://dev.ai-generated.dev",
        "https://www.ai-generated.dev",
        "https://ai-generated.dev",
        "https://www.ai-generated.fr",
        "https://ai-generated.fr",
        "https://www.ai-generated.fr:8080",
        "https://ai-generated.fr:8080",
        "https://localhost:8443",
        "https://localhost:3443",
        "http://localhost:3000",
    ];
}
