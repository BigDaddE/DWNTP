use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub async fn send<W: AsyncWriteExt + Unpin, T: serde::Serialize>(w: &mut W, msg: &T) -> Result<()> {
    let json = serde_json::to_vec(msg)?;
    w.write_all(&(json.len() as u32).to_be_bytes()).await.context("write len")?;
    w.write_all(&json).await.context("write body")?;
    Ok(())
}

pub async fn recv<R: AsyncReadExt + Unpin, T: serde::de::DeserializeOwned>(r: &mut R) -> Result<T> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf).await.context("read len")?;
    let mut body = vec![0u8; u32::from_be_bytes(len_buf) as usize];
    r.read_exact(&mut body).await.context("read body")?;
    Ok(serde_json::from_slice(&body)?)
}
