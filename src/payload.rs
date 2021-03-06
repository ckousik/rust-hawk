use ring::digest;

/// A utility for hashing payloads. Feed your entity body to this, then pass the `finish`
/// result to a request or response.
pub struct PayloadHasher {
    context: digest::Context,
    algorithm: &'static digest::Algorithm,
}

impl PayloadHasher {
    /// Create a new PayloadHasher. The `content_type` should be lower-case and should
    /// not include parameters. The digest is assumed to be the same as the digest used
    /// for the credentials in the request.
    pub fn new<B>(content_type: B, algorithm: &'static digest::Algorithm) -> Self
        where B: AsRef<[u8]>
    {
        let mut hasher = PayloadHasher {
            context: digest::Context::new(algorithm),
            algorithm,
        };
        hasher.update(b"hawk.1.payload\n");
        hasher.update(content_type.as_ref());
        hasher.update(b"\n");
        hasher
    }

    /// Hash a single value and return it
    pub fn hash<B1, B2>(content_type: B1,
                        algorithm: &'static digest::Algorithm,
                        payload: B2)
                        -> Vec<u8>
        where B1: AsRef<[u8]>,
              B2: AsRef<[u8]>
    {
        let mut hasher = PayloadHasher::new(content_type, algorithm);
        hasher.update(payload);
        hasher.finish()
    }

    /// Update the hash with new data.
    pub fn update<B>(&mut self, data: B)
        where B: AsRef<[u8]>
    {
        self.context.update(data.as_ref());
    }

    /// Finish hashing and return the result
    ///
    /// Note that this appends a newline to the payload, as does the JS Hawk implementaiton.
    pub fn finish(mut self) -> Vec<u8> {
        self.update(b"\n");
        let digest = self.context.finish();
        let mut rv = vec![0; self.algorithm.output_len];
        rv.clone_from_slice(digest.as_ref());
        rv
    }
}

#[cfg(test)]
mod tests {
    use super::PayloadHasher;
    use ring::digest::SHA256;

    #[test]
    fn hash_consistency() {
        let mut hasher1 = PayloadHasher::new("text/plain", &SHA256);
        hasher1.update("pày");
        hasher1.update("load");
        let hash1 = hasher1.finish();

        let mut hasher2 = PayloadHasher::new("text/plain", &SHA256);
        hasher2.update("pàyload");
        let hash2 = hasher2.finish();

        let hash3 = PayloadHasher::hash("text/plain", &SHA256, "pàyload");

        let hash4 = // "pàyload" as utf-8 bytes
            PayloadHasher::hash("text/plain", &SHA256, vec![112, 195, 160, 121, 108, 111, 97, 100]);

        assert_eq!(hash1,
                   vec![228, 238, 241, 224, 235, 114, 158, 112, 211, 254, 118, 89, 25, 236, 87,
                        176, 181, 54, 61, 135, 42, 223, 188, 103, 194, 59, 83, 36, 136, 31, 198,
                        50]);
        assert_eq!(hash2, hash1);
        assert_eq!(hash3, hash1);
        assert_eq!(hash4, hash1);
    }
}
