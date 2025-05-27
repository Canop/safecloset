use {
    aes_gcm_siv::Nonce,
    rand::{
        Rng,
        RngCore,
        rng,
    },
    std::ops::Range,
};

const PASSWORD_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789)(*&^%$#@!~\
                                ._[]{}/;:?%,=-+'";

pub fn random_bytes(count: usize) -> Box<[u8]> {
    let mut vec = vec![0u8; count];
    rng().fill_bytes(&mut vec);
    vec.into_boxed_slice()
}

/// Generate a random length array of random bytes.
///
/// min_size and max_size are both included.
#[allow(dead_code)]
pub fn random_bytes_random_size(range: Range<usize>) -> Box<[u8]> {
    random_bytes(rng().random_range(range))
}

/// Generate a random nonce for AES-GCM.
///
/// AES-GCM nonces are 12 bytes (96 bits)
pub fn random_nonce() -> Nonce {
    let mut nonce = Nonce::default();
    rng().fill_bytes(&mut nonce[0..12]);
    nonce
}

pub fn random_password() -> String {
    let mut rng = rng();
    (0..rng.random_range(30..80))
        .map(|_| {
            let idx = rng.random_range(0..PASSWORD_CHARSET.len());
            PASSWORD_CHARSET[idx] as char
        })
        .collect()
}

/// check we're really making different nonces
#[test]
pub fn test_random_nonce() {
    assert_ne!(random_nonce(), random_nonce());
}
