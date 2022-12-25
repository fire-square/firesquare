//! Network utilities.

use std::path::Path;

use reqwest::Client;
use thiserror::Error;
use tokio::{fs, io::AsyncWriteExt};

/// Network error.
#[derive(Error, Debug)]
pub enum NetworkError {
	/// Network error. See [`reqwest::Error`] for details.
	#[error("Network error: {0}")]
	NetworkError(#[from] reqwest::Error),
	/// IO error. See [`tokio::io::Error`] for details.
	#[error("IO error: {0}")]
	IOError(#[from] tokio::io::Error),
	/// Directory not exists error.
	#[error("Directory not exists: {0}")]
	DirectoryNotExists(String),
}

/// Network client.
///
/// This is a wrapper around [`reqwest::Client`] and functions in this module.
pub struct NetClient {
	client: Client,
}

impl NetClient {
	/// Creates a new network client.
	pub fn new() -> Self {
		Self {
			client: Client::new(),
		}
	}

	/// Creates a new network client from the given [`reqwest::Client`].
	pub fn from_client(client: Client) -> Self {
		Self { client }
	}

	/// Returns a reference to the underlying [`reqwest::Client`].
	pub fn client(&self) -> &Client {
		&self.client
	}

	/// Downloads a file from the given URL to the given path.
	///
	/// See [`download_to`] for details.
	pub async fn download_to(&self, url: &str, path: &Path) -> Result<(), NetworkError> {
		download_to(&self.client, url, path).await
	}
}

impl Default for NetClient {
	fn default() -> Self {
		Self::new()
	}
}

/// Downloads a file from the given URL to the given path.
///
/// Function downloads a file from the given URL to the given path.
/// If the file already exists, it will be overwritten.
///
/// It chunks the file to not use too much memory.
///
/// # Examples
///
/// ```
/// use firesquare_launcher::utils::net::download_to;
/// use tokio::runtime::Runtime;
/// use std::path::Path;
///
/// let mut rt = Runtime::new().unwrap();
/// rt.block_on(async {
///   download_to(&reqwest::Client::new(), "https://ipfs.frsqr.xyz/ipfs/bafybeih764jjsjnf5inznxgifpzuzinhgn4565sxxqtl2vuylaawc6mzf4/hello.txt", &Path::new("hello.txt")).await.unwrap();
/// });
///
/// // Check that the file was downloaded
/// assert!(Path::new("hello.txt").exists());
///
/// // Cleanup
/// std::fs::remove_file("hello.txt").unwrap();
/// ```
///
/// # Errors
///
/// - [`NetworkError::NetworkError`] if there was an error while downloading the file.
/// - [`NetworkError::IOError`] if there was an error while writing the file.
/// - [`NetworkError::DirectoryNotExists`] if the parent directory of the given path does not exist.
pub async fn download_to(client: &Client, url: &str, path: &Path) -> Result<(), NetworkError> {
	if path.parent().is_none() {
		return Err(NetworkError::DirectoryNotExists(
			path.to_str().unwrap().to_string(),
		));
	}
	let mut response = client.get(url).send().await?;
	let mut file = fs::File::create(path).await?;
	while let Some(chunk) = response.chunk().await? {
		file.write_all(&chunk).await?;
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::path::Path;

	#[tokio::test]
	async fn test_download_to() {
		let client = Client::new();
		download_to(&client, "https://ipfs.frsqr.xyz/ipfs/bafybeih764jjsjnf5inznxgifpzuzinhgn4565sxxqtl2vuylaawc6mzf4/hello.txt", &Path::new("hello.txt")).await.unwrap();

		// Check that the file was downloaded
		assert!(Path::new("hello.txt").exists());

		// Cleanup
		std::fs::remove_file("hello.txt").unwrap();
	}
}
