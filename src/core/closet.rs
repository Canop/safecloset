use {
    super::*,
    aes_gcm_siv::{
        Aes256GcmSiv,
        Key,
        KeyInit,
    },
    rand::{
        Rng,
        distr::{
            Distribution,
            weighted::WeightedIndex,
        },
        rng,
        seq::SliceRandom,
    },
    serde::{
        Deserialize,
        Serialize,
    },
    std::{
        fs,
        io::BufWriter,
        path::{
            Path,
            PathBuf,
        },
    },
};

/// The closet containing all the crypted drawers
#[derive(Serialize, Deserialize)]
pub struct Closet {
    /// Clear comments, which can be read with a standard binary/hex editor
    #[serde(default = "default_clear_comments")]
    pub comments: String,

    /// The salt used to generate the cipher keys from the passwords
    pub salt: String,

    /// The crypted drawers
    pub drawers: Vec<ClosedDrawer>,
}

/// Return the default comments which are written in the clear
/// part of the file. The rationale for writing the name of the
/// soft is that you may forget it and need to find the software
/// to open your closet file, while attackers are the ones who
/// may the most easily guess it.
fn default_clear_comments() -> String {
    "Closet file written with SafeCloset\nhttps://dystroy.org/safecloset".to_string()
}

/// Remove all permissions but the owner's, on systems having them
#[cfg(unix)]
fn restrict_permissions(path: &Path) -> Result<(), CoreError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) -> Result<(), CoreError> {
    Ok(())
}

/// Path of the file the closet is written to before it replaces the
/// previous version
fn writing_path(path: &Path) -> PathBuf {
    let mut file_name = path.file_name().unwrap_or_default().to_os_string();
    file_name.push(".writing");
    path.with_file_name(file_name)
}

/// Make a rename durable, on systems where a directory can be synced.
///
/// Failing to sync isn't worth failing the save, as the content is already
/// written and renamed.
#[cfg(unix)]
fn sync_dir(path: &Path) {
    let dir = match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        _ => Path::new("."),
    };
    if let Err(e) = fs::File::open(dir).and_then(|dir| dir.sync_all()) {
        warn!("can't sync directory: {e}");
    }
}

#[cfg(not(unix))]
fn sync_dir(_path: &Path) {}

/// Relative frequencies of the possible numbers of decoy drawers of a
/// closet, by depth: the weight at index i is the frequency of i drawers.
///
/// The weight of 0 is null, so that a drawer is never alone in a closet,
/// and there's no line beyond the last depth, which bounds the tree.
const DECOY_DRAWERS_WEIGHTS: &[&[u32]] = &[
    &[0, 0, 0, 520, 260, 120, 52, 24, 11, 5, 3, 2, 1, 1, 1],
    &[0, 622, 253, 83, 25, 9, 4, 2, 1, 1],
    &[0, 762, 178, 40, 12, 5, 2, 1],
    &[425, 533, 28, 9, 3, 1, 1],
    &[824, 165, 8, 2, 1],
];

/// compute the number of decoy drawers we must create for
/// the given depth
fn random_decoy_drawers_count(
    rng: &mut impl Rng,
    depth: usize,
) -> usize {
    match DECOY_DRAWERS_WEIGHTS.get(depth) {
        Some(weights) => WeightedIndex::new(weights.iter())
            .expect("valid decoy drawers weights")
            .sample(rng),
        None => 0,
    }
}

impl Closet {
    pub fn new(depth: usize) -> Result<Self, CoreError> {
        let comments = default_clear_comments();
        let salt = random_password();
        let drawers = Vec::new();
        let mut closet = Self {
            comments,
            salt,
            drawers,
        };
        // creating decoy drawers
        let decoy_drawers_count = random_decoy_drawers_count(&mut rng(), depth);
        for _ in 0..decoy_drawers_count {
            closet.create_drawer_unchecked(depth, random_password())?;
        }
        Ok(closet)
    }

    /// Save the closet to a file.
    pub fn save(
        &mut self,
        path: &Path,
    ) -> Result<(), CoreError> {
        self.shuffle_drawers();
        let writing_path = writing_path(path);
        if writing_path.exists() {
            fs::remove_file(&writing_path)?;
        }
        if let Err(e) = self.write_to_file(&writing_path) {
            let _ = fs::remove_file(&writing_path);
            return Err(e);
        }
        if path.exists() {
            let backup_path = path.with_extension("old");
            if backup_path.exists() {
                fs::remove_file(&backup_path)?;
            }
            fs::copy(path, &backup_path)?;
            // the backup may come from a version not restricting permissions
            restrict_permissions(&backup_path)?;
        }
        fs::rename(&writing_path, path)?;
        sync_dir(path);
        Ok(())
    }

    pub fn write_to_file(
        &self,
        path: &Path,
    ) -> Result<(), CoreError> {
        if path.exists() {
            return Err(CoreError::FileExists(path.to_path_buf()));
        }
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(path)?;
        // the serialization does very small writes, so buffering is what
        // makes saving fast
        let mut writer = BufWriter::new(file);
        rmp_serde::encode::write_named(&mut writer, &self)?;
        // not letting the writer be dropped, as it would hide a failure
        // of the last flush
        let file = writer.into_inner().map_err(|e| e.into_error())?;
        file.sync_all()?;
        Ok(())
    }

    /// read a closet from a file
    pub fn from_file(path: &Path) -> Result<Self, CoreError> {
        let file = fs::File::open(path)?;
        let closet = rmp_serde::decode::from_read(file)?;
        Ok(closet)
    }

    /// Create a drawer without checking first the password isn't used by
    /// another drawer, or that the password meets minimal requirements,
    /// add it to the closed drawers of the closet.
    ///
    /// This is fast but dangerous, and should not be used on user action.
    fn create_drawer_unchecked(
        &mut self,
        depth: usize,
        password: String,
    ) -> Result<OpenDrawer, CoreError> {
        let drawer_content = DrawerContent::new(depth)?;
        let mut open_drawer = OpenDrawer::new(depth, password, drawer_content);
        let closed_drawer = open_drawer.close(self)?;
        self.drawers.push(closed_drawer);
        Ok(open_drawer)
    }

    /// Create a drawer, add it to the closet.
    ///
    /// Return an error if the password is already used by
    /// another drawer (which probably means the user wanted
    /// to open a drawer and not create one).
    pub fn create_drawer(
        &mut self,
        depth: usize,
        password: String,
    ) -> Result<OpenDrawer, CoreError> {
        if password.len() < MIN_PASSWORD_LENGTH {
            return Err(CoreError::PasswordTooShort);
        }
        if self.is_password_taken(depth, &password) {
            return Err(CoreError::PasswordAlreadyUsed);
        }
        self.create_drawer_unchecked(depth, password)
    }

    /// Open the drawer responding to this password and return it.
    ///
    /// Return None when no drawer can be opened with this password.
    pub fn open_drawer(
        &self,
        depth: usize,
        password: &str,
    ) -> Option<OpenDrawer> {
        if self.drawers.is_empty() {
            return None;
        }
        let cipher = time!("cipher", self.cipher(password)).ok()?;
        for closed_drawer in &self.drawers {
            if let Ok(open_drawer) = closed_drawer.open(depth, password, &cipher) {
                return Some(open_drawer);
            }
        }
        None
    }

    pub fn is_password_taken(
        &self,
        depth: usize,
        password: &str,
    ) -> bool {
        self.open_drawer(depth, password).is_some()
    }

    /// Close the passed drawer, put it back among closed ones
    pub fn close_drawer(
        &mut self,
        mut open_drawer: OpenDrawer,
    ) -> Result<bool, CoreError> {
        let closed_drawer = open_drawer.close(self)?;
        Ok(self.push_drawer_back(closed_drawer))
    }

    fn push_drawer_back(
        &mut self,
        drawer: ClosedDrawer,
    ) -> bool {
        for idx in 0..self.drawers.len() {
            if self.drawers[idx].has_same_id(&drawer) {
                self.drawers[idx] = drawer;
                return true;
            }
        }
        false
    }

    /// Change the order of drawers
    pub fn shuffle_drawers(&mut self) {
        self.drawers.shuffle(&mut rng());
    }

    /// Close the drawer then reopen it.
    ///
    /// After this operation, the closet contains the content of the given
    /// drawer but the closet isn't saved.
    #[allow(dead_code)]
    pub fn close_then_reopen(
        &mut self,
        drawer: OpenDrawer,
    ) -> Result<OpenDrawer, CoreError> {
        let depth = drawer.depth;
        let password = drawer.password.clone();
        self.close_drawer(drawer)?;
        self.open_drawer(depth, &password).ok_or_else(|| {
            // shouldn't happen
            CoreError::InternalError("can't reopen just closed drawer".to_string())
        })
    }

    pub fn cipher(
        &self,
        password: &str,
    ) -> Result<Aes256GcmSiv, CoreError> {
        let config = argon2::Config {
            ad: &[],
            hash_length: 32,
            lanes: 1,
            mem_cost: 4096,
            secret: &[],
            thread_mode: argon2::ThreadMode::default(),
            time_cost: 3,
            variant: argon2::Variant::Argon2i,
            version: argon2::Version::Version13,
        };
        let hash = argon2::hash_raw(password.as_bytes(), self.salt.as_bytes(), &config)?;
        let key = Key::<Aes256GcmSiv>::from_slice(&hash);
        Ok(Aes256GcmSiv::new(key))
    }
}

#[cfg(test)]
mod decoy_drawers_tests {

    use {
        super::*,
        termimad::ProgressBar,
    };

    /// Number of drawers a closet created at the given depth would have,
    /// the ones of its deeper closets included
    fn simulate_drawers_count(
        rng: &mut impl Rng,
        depth: usize,
    ) -> usize {
        let count = random_decoy_drawers_count(rng, depth);
        let mut total = count;
        for _ in 0..count {
            total += simulate_drawers_count(rng, depth + 1);
        }
        total
    }

    /// Bytes a drawer of the given depth takes in the file, for the given
    /// size of its own content.
    ///
    /// Byte arrays are serialized as arrays of integers, which costs about
    /// 1.5 byte per byte, and a drawer is nested in the arrays of all its
    /// ancestors, so the cost compounds with the depth.
    fn drawer_file_bytes(
        content_size: usize,
        depth: usize,
    ) -> usize {
        let mut bytes = content_size;
        for _ in 0..=depth {
            bytes = bytes * 3 / 2;
        }
        bytes
    }

    /// Bytes the decoy drawers of a closet created at the given depth would
    /// take in the file, the ones of its deeper closets included.
    ///
    /// Only accounts for the drawers, not for the few dozen bytes of each
    /// closet.
    fn simulate_file_bytes(
        rng: &mut impl Rng,
        depth: usize,
    ) -> usize {
        // nonce, id, tag, keys and framing of a drawer holding no entry
        const DRAWER_OVERHEAD: usize = 130;

        let count = random_decoy_drawers_count(rng, depth);
        let mut total = 0;
        for _ in 0..count {
            let content_size = rng.random_range(GARBAGE_SIZE_RANGE) + DRAWER_OVERHEAD;
            total += drawer_file_bytes(content_size, depth) + simulate_file_bytes(rng, depth + 1);
        }
        total
    }

    /// check a new closet can't have an absurd number of drawers
    #[test]
    fn test_decoy_drawers_count() {
        const CLOSETS: usize = 1000;
        // 3 root drawers, each holding a chain of 2 deeper drawers
        const MIN: usize = 9;
        // far above the ~130 biggest of 100 000 simulated closets
        const MAX: usize = 500;

        let mut rng = rng();
        let mut sum = 0;
        for _ in 0..CLOSETS {
            let count = simulate_drawers_count(&mut rng, 0);
            assert!(
                (MIN..=MAX).contains(&count),
                "a new closet would have {count} drawers",
            );
            sum += count;
        }
        let mean = sum as f64 / CLOSETS as f64;
        assert!(
            (20.0..30.0).contains(&mean),
            "the mean number of drawers of a new closet is {mean:.2}",
        );
    }

    /// Print a histogram of the given values
    fn print_histogram(values: &[usize]) {
        const BUCKETS: usize = 40;
        const BAR_WIDTH: usize = 60;

        let Some(&min) = values.iter().min() else {
            return;
        };
        let max = *values.iter().max().unwrap();
        let bucket_width = (max - min) / BUCKETS + 1;
        let mut buckets = vec![0; (max - min) / bucket_width + 1];
        for &value in values {
            buckets[(value - min) / bucket_width] += 1;
        }
        let highest = buckets.iter().copied().max().unwrap().max(1);
        // an eighth of a character, so that a rare bucket isn't empty
        let min_part = 1.0 / (8 * BAR_WIDTH) as f32;
        for (idx, &count) in buckets.iter().enumerate() {
            let low = min + idx * bucket_width;
            let label = if bucket_width == 1 {
                format!("{low:5}")
            } else {
                format!("{low:5}-{:<5}", low + bucket_width - 1)
            };
            let part = if count == 0 {
                0.0
            } else {
                (count as f32 / highest as f32).max(min_part)
            };
            println!(
                "  {label} |{:<width$}| {:.2}%",
                ProgressBar::new(part, BAR_WIDTH),
                100.0 * count as f64 / values.len() as f64,
                width = BAR_WIDTH,
            );
        }
    }

    /// Print the stats and the histogram of the given values
    fn print_distribution(
        title: &str,
        values: &mut [usize],
    ) {
        values.sort_unstable();
        let count = values.len();
        let mean = values.iter().sum::<usize>() as f64 / count as f64;
        let variance = values
            .iter()
            .map(|&value| (value as f64 - mean).powi(2))
            .sum::<f64>()
            / count as f64;
        println!("{title}");
        println!("  mean:   {mean:.2}");
        println!("  stddev: {:.2}", variance.sqrt());
        println!("  min:    {}", values[0]);
        for (label, permille) in [("p50", 500), ("p90", 900), ("p99", 990), ("p99.9", 999)] {
            println!("  {label}:    {}", values[count * permille / 1000]);
        }
        println!("  max:    {}", values[count - 1]);
        print_histogram(values);
    }

    /// Print the distribution of the number of decoy drawers of new closets,
    /// and of the bytes they take in the file.
    ///
    /// ```not_rust
    /// cargo test --release observe_decoys -- --ignored --nocapture
    /// ```
    #[test]
    #[ignore = "observation, run explicitly"]
    fn observe_decoys() {
        const CLOSETS: usize = 100_000;

        let mut rng = rng();
        let mut counts: Vec<usize> = (0..CLOSETS)
            .map(|_| simulate_drawers_count(&mut rng, 0))
            .collect();
        let mut kilobytes: Vec<usize> = (0..CLOSETS)
            .map(|_| simulate_file_bytes(&mut rng, 0) / 1024)
            .collect();
        print_distribution(&format!("drawers of {CLOSETS} closets:"), &mut counts);
        print_distribution(&format!("kilobytes of {CLOSETS} closets:"), &mut kilobytes);

        // real closets, to check the estimation isn't far off
        const REAL_CLOSETS: usize = 20;
        let temp_dir = tempfile::tempdir().unwrap();
        let mut real_kilobytes: Vec<usize> = (0..REAL_CLOSETS)
            .map(|i| {
                let path = temp_dir.path().join(format!("observed-{i}.closet"));
                let mut open_closet = OpenCloset::create(path.to_path_buf()).unwrap();
                open_closet.close_and_save().unwrap();
                std::fs::metadata(&path).unwrap().len() as usize / 1024
            })
            .collect();
        real_kilobytes.sort_unstable();
        println!(
            "kilobytes of {REAL_CLOSETS} real closets: min {}, median {}, max {}",
            real_kilobytes[0],
            real_kilobytes[REAL_CLOSETS / 2],
            real_kilobytes[REAL_CLOSETS - 1],
        );
        temp_dir.close().unwrap();
    }
}
