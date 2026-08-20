use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("网络请求失败: {0}")]
    Http(String),

    #[error("接口返回了无效 JSON: {0}")]
    Json(#[from] serde_json::Error),

    #[error("未找到匹配器件: {0}")]
    NotFound(String),

    #[error("该器件没有 3D 模型")]
    No3dModel,

    #[error("该器件没有原理图或 PCB 封装")]
    NoSymbolOrFootprint,

    #[error("写出 Altium 库失败: {0}")]
    Altium(String),

    #[error(transparent)]
    Io(#[from] io::Error),
}

impl Error {
    pub fn msg(text: impl Into<String>) -> Self {
        Self::Message(text.into())
    }
}
