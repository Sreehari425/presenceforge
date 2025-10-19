//! Runtime-agnostic async I/O traits
//!
//! These traits provide a common interface for async I/O operations
//! that can be implemented by different async runtimes.

use std::future::Future;
use std::io;
use std::pin::Pin;

/// Asynchronous version of std::io::Read
///
/// This trait defines the interface for asynchronous read operations.
/// It is designed to be runtime-agnostic and can be implemented for
/// any async runtime's types (e.g., tokio::net::TcpStream, async_std::net::TcpStream).
pub trait AsyncRead {
    /// Read bytes asynchronously into the buffer
    ///
    /// Returns a future that resolves to the number of bytes read or an I/O error.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to read into
    ///
    /// # Returns
    ///
    /// A future that resolves to the number of bytes read
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>>;
}

/// Default implementation of read_exact using AsyncRead
pub async fn read_exact<T: AsyncRead + Unpin + ?Sized>(
    reader: &mut T,
    mut buf: &mut [u8],
) -> io::Result<()> {
    while !buf.is_empty() {
        match reader.read(buf).await {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "failed to fill buffer",
                ));
            }
            Ok(n) => buf = &mut buf[n..],
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Asynchronous version of std::io::Write
///
/// This trait defines the interface for asynchronous write operations.
/// It is designed to be runtime-agnostic and can be implemented for
/// any async runtime's types.
pub trait AsyncWrite {
    /// Write bytes asynchronously from the buffer
    ///
    /// Returns a future that resolves to the number of bytes written or an I/O error.
    ///
    /// # Arguments
    ///
    /// * `buf` - The buffer to write from
    ///
    /// # Returns
    ///
    /// A future that resolves to the number of bytes written
    fn write<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = io::Result<usize>> + Send + 'a>>;

    /// Flush the writer asynchronously
    ///
    /// # Returns
    ///
    /// A future that resolves when the flush is complete
    fn flush<'a>(&'a mut self) -> Pin<Box<dyn Future<Output = io::Result<()>> + Send + 'a>>;
}

/// Default implementation of write_all using AsyncWrite
pub async fn write_all<T: AsyncWrite + Unpin + ?Sized>(
    writer: &mut T,
    mut buf: &[u8],
) -> io::Result<()> {
    while !buf.is_empty() {
        match writer.write(buf).await {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write whole buffer",
                ));
            }
            Ok(n) => buf = &buf[n..],
            Err(e) => return Err(e),
        }
    }
    writer.flush().await
}

/// Utility functions for async IPC operations
pub mod ipc_utils {
    use super::*;

    /// Read a little-endian u32 value
    pub async fn read_u32_le<T: AsyncRead + Unpin>(reader: &mut T) -> io::Result<u32> {
        let mut buffer = [0u8; 4];
        super::read_exact(reader, &mut buffer).await?;
        Ok(u32::from_le_bytes(buffer))
    }

    /// Write a little-endian u32 value
    #[allow(dead_code)]
    pub async fn write_u32_le<T: AsyncWrite + Unpin>(writer: &mut T, value: u32) -> io::Result<()> {
        super::write_all(writer, &value.to_le_bytes()).await
    }

    /// Read a Discord IPC frame header
    #[allow(dead_code)]
    pub async fn read_header<T: AsyncRead + Unpin>(reader: &mut T) -> io::Result<(u32, u32)> {
        let opcode = read_u32_le(reader).await?;
        let length = read_u32_le(reader).await?;
        Ok((opcode, length))
    }
}
