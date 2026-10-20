use std::ops::Deref;
use zeroize::Zeroize;

/// A wrapper around sensitive byte data that zeroizes memory on drop.
///
/// This prevents secret material from persisting in process memory after use.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SecretBytes {
    inner: Vec<u8>,
}

impl SecretBytes {
    pub fn new(data: Vec<u8>) -> Self {
        Self { inner: data }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.inner
    }
}

impl AsRef<[u8]> for SecretBytes {
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl Deref for SecretBytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.inner
    }
}

/// Zeroize a buffer manually (for cases where SecretBytes wrapper can't be used).
pub fn zeroize_buffer(buf: &mut Vec<u8>) {
    buf.zeroize();
}

/// Securely compare two byte slices in constant time to prevent timing attacks.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}
