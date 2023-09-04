use enter_core_lib::engine::{
    Engine, EngineConfig, EngineExecError, EngineInitError, EngineLoopMode, EngineTargetFps,
};
use pollster::FutureExt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("engine init error: {0}")]
    EngineInitError(#[from] EngineInitError),
    #[error("engine exec error: {0}")]
    EngineExecError(#[from] EngineExecError),
}

fn main() -> Result<(), Error> {
    Engine::new(EngineConfig {
        title: format!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        resizable: false,
        width: 800,
        height: 600,
    })
    .block_on()?
    .run(EngineLoopMode::Poll, EngineTargetFps::VSync)?;
    Ok(())
}
