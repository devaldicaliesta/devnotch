//! Rak berkas (File Shelf): berkas yang diseret ke Island disimpan sementara sebagai rujukan,
//! lalu bisa diseret keluar ke folder atau aplikasi lain.
//!
//! Hanya jalurnya yang disimpan; berkas tidak pernah disalin atau dipindahkan oleh Notchi.

use std::path::{Path, PathBuf};

pub const MAX_ITEMS: usize = 24;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub path: PathBuf,
    pub name: String,
}

impl Item {
    fn new(path: PathBuf) -> Item {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        Item { path, name }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Shelf {
    items: Vec<Item>,
}

fn same(a: &Path, b: &Path) -> bool {
    a.as_os_str().to_string_lossy().eq_ignore_ascii_case(&b.as_os_str().to_string_lossy())
}

impl Shelf {
    pub fn new() -> Self {
        Self::default()
    }

    /// Tambahkan berkas; yang sudah ada dilewati, dan jumlahnya dibatasi [`MAX_ITEMS`]
    /// (yang paling lama dibuang). Mengembalikan jumlah yang benar-benar ditambahkan.
    pub fn add(&mut self, paths: impl IntoIterator<Item = PathBuf>) -> usize {
        let mut added = 0;
        for p in paths {
            if p.as_os_str().is_empty() || self.items.iter().any(|i| same(&i.path, &p)) {
                continue;
            }
            self.items.push(Item::new(p));
            added += 1;
        }
        if self.items.len() > MAX_ITEMS {
            let extra = self.items.len() - MAX_ITEMS;
            self.items.drain(..extra);
        }
        added
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn paths(&self) -> Vec<PathBuf> {
        self.items.iter().map(|i| i.path.clone()).collect()
    }

    pub fn get(&self, i: usize) -> Option<&Item> {
        self.items.get(i)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Keluarkan berkas tertentu dari rak (misalnya setelah berhasil diseret ke aplikasi lain).
    pub fn remove_paths(&mut self, paths: &[PathBuf]) {
        self.items.retain(|i| !paths.iter().any(|p| same(&i.path, p)));
    }

    /// Buang berkas yang sudah tidak ada (misalnya dipindahkan saat diseret keluar).
    pub fn prune(&mut self, exists: impl Fn(&Path) -> bool) -> bool {
        let before = self.items.len();
        self.items.retain(|i| exists(&i.path));
        self.items.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pb(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn tambah_tanpa_duplikat() {
        let mut s = Shelf::new();
        assert_eq!(s.add([pb("C:/a/Laporan.pdf"), pb("C:/a/foto.png")]), 2);
        assert_eq!(s.add([pb("c:/A/laporan.PDF"), pb("")]), 0, "jalur Windows tidak peka huruf besar");
        assert_eq!(s.items()[0].name, "Laporan.pdf");
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn dibatasi_dan_yang_lama_dibuang() {
        let mut s = Shelf::new();
        s.add((0..MAX_ITEMS + 3).map(|i| pb(&format!("C:/f{i}.txt"))));
        assert_eq!(s.len(), MAX_ITEMS);
        assert_eq!(s.items()[0].name, "f3.txt");
    }

    #[test]
    fn buang_yang_hilang() {
        let mut s = Shelf::new();
        s.add([pb("C:/ada.txt"), pb("C:/pindah.txt")]);
        assert!(s.prune(|p| p.ends_with("ada.txt")));
        assert_eq!(s.paths(), vec![pb("C:/ada.txt")]);
        assert!(!s.prune(|_| true));
        s.add([pb("C:/b.txt"), pb("C:/c.txt")]);
        s.remove_paths(&[pb("c:/B.TXT")]);
        assert_eq!(s.paths(), vec![pb("C:/ada.txt"), pb("C:/c.txt")]);
        s.clear();
        assert!(s.is_empty() && s.get(0).is_none());
    }
}
