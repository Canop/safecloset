use {
    aes_gcm_siv::Nonce,
    rand::{
        thread_rng,
        Rng,
        RngCore,
    },
    std::ops::Range,
};

const PASSWORD_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                abcdefghijklmnopqrstuvwxyz\
                                0123456789)(*&^%$#@!~\
                                ._[]{}/;:?%,=-+'";

pub fn random_bytes(count: usize) -> Box<[u8]> {
    let mut vec = vec![0u8; count];
    thread_rng().fill_bytes(&mut vec);
    vec.into_boxed_slice()
}

/// Generate a random length array of random bytes.
///
/// min_size and max_size are both included.
#[allow(dead_code)]
pub fn random_bytes_random_size(range: Range<usize>) -> Box<[u8]> {
    random_bytes(thread_rng().gen_range(range))
}

/// Generate a random nonce for AES-GCM.
///
/// AES-GCM nonces are 12 bytes (96 bits)
pub fn random_nonce() -> Nonce {
    let mut nonce = Nonce::default();
    thread_rng().fill_bytes(&mut nonce[0..12]);
    nonce
}

pub fn random_password() -> String {
    let mut rng = thread_rng();
    (0..rng.gen_range(30..80))
        .map(|_| {
            let idx = rng.gen_range(0..PASSWORD_CHARSET.len());
            PASSWORD_CHARSET[idx] as char
        })
        .collect()
}

/// check we're really making different nonces
#[test]
pub fn test_random_nonce() {
    assert_ne!(random_nonce(), random_nonce());
}
