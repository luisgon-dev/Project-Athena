use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{
    db::repositories::SqliteRequestRepository,
    domain::{imports::ImportMediaType, library::ScannedItem, requests::MediaType},
    importer::classify::classify_payload,
};

pub struct LibraryScanner;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScanCounts {
    pub ebooks_found: i64,
    pub audiobooks_found: i64,
    pub duplicates_skipped: i64,
}

impl LibraryScanner {
    pub async fn scan(
        ebooks_root: &Path,
        audiobooks_root: &Path,
        repo: &SqliteRequestRepository,
    ) -> Result<ScanCounts> {
        let mut counts = ScanCounts::default();

        if ebooks_root.exists() {
            Self::scan_root(ebooks_root, MediaType::Ebook, repo, &mut counts).await?;
        }
        if audiobooks_root.exists() {
            Self::scan_root(audiobooks_root, MediaType::Audiobook, repo, &mut counts).await?;
        }

        Ok(counts)
    }

    async fn scan_root(
        root: &Path,
        media_type: MediaType,
        repo: &SqliteRequestRepository,
        counts: &mut ScanCounts,
    ) -> Result<()> {
        for author_entry in std::fs::read_dir(root)? {
            let author_entry = author_entry?;
            let author_path = author_entry.path();
            if !author_path.is_dir() {
                continue;
            }

            for title_entry in std::fs::read_dir(&author_path)? {
                let title_entry = title_entry?;
                let title_path = title_entry.path();
                if !title_path.is_dir() {
                    continue;
                }

                let files = collect_files_recursively(&title_path)?;
                if files.is_empty() {
                    continue;
                }

                let file_strings: Vec<String> =
                    files.iter().map(|p| p.display().to_string()).collect();
                let classification = classify_payload(&file_strings);

                let expected_media = match media_type {
                    MediaType::Ebook => ImportMediaType::Ebook,
                    MediaType::Audiobook => ImportMediaType::Audiobook,
                };

                if classification.media_type != expected_media
                    && classification.media_type != ImportMediaType::Mixed
                {
                    continue;
                }

                let author = author_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown Author")
                    .to_string();
                let title = title_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown Title")
                    .to_string();
                let imported_path = title_path.display().to_string();

                let path_duplicate = repo
                    .find_request_by_imported_path(&imported_path, media_type.as_str())
                    .await?;
                let title_duplicate = repo
                    .find_request_by_title_author(&title, &author, media_type.as_str())
                    .await?;

                if path_duplicate.is_some() || title_duplicate.is_some() {
                    counts.duplicates_skipped += 1;
                    continue;
                }

                repo.create_library_discovered_request(&ScannedItem {
                    author,
                    title,
                    media_type: media_type.clone(),
                    imported_path,
                })
                .await?;

                match media_type {
                    MediaType::Ebook => counts.ebooks_found += 1,
                    MediaType::Audiobook => counts.audiobooks_found += 1,
                }
            }
        }

        Ok(())
    }
}

fn collect_files_recursively(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_files_recursively(&path)?);
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_repo() -> SqliteRequestRepository {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        SqliteRequestRepository::new(pool)
    }

    #[tokio::test]
    async fn scanner_discovers_ebook_and_audiobook() {
        let repo = setup_repo().await;
        let tmp = tempfile::tempdir().unwrap();
        let ebooks = tmp.path().join("ebooks");
        let audiobooks = tmp.path().join("audiobooks");

        std::fs::create_dir_all(ebooks.join("Isaac Asimov").join("Foundation")).unwrap();
        std::fs::write(
            ebooks
                .join("Isaac Asimov")
                .join("Foundation")
                .join("Foundation.epub"),
            "",
        )
        .unwrap();

        std::fs::create_dir_all(audiobooks.join("Frank Herbert").join("Dune")).unwrap();
        std::fs::write(
            audiobooks
                .join("Frank Herbert")
                .join("Dune")
                .join("part1.m4b"),
            "",
        )
        .unwrap();

        let counts = LibraryScanner::scan(&ebooks, &audiobooks, &repo)
            .await
            .unwrap();

        assert_eq!(counts.ebooks_found, 1);
        assert_eq!(counts.audiobooks_found, 1);
        assert_eq!(counts.duplicates_skipped, 0);
    }

    #[tokio::test]
    async fn scanner_skips_duplicates_by_path_and_title_author() {
        let repo = setup_repo().await;
        let tmp = tempfile::tempdir().unwrap();
        let ebooks = tmp.path().join("ebooks");

        std::fs::create_dir_all(ebooks.join("Author One").join("Book One")).unwrap();
        std::fs::write(
            ebooks.join("Author One").join("Book One").join("book.epub"),
            "",
        )
        .unwrap();

        // First scan should succeed
        let counts1 = LibraryScanner::scan(&ebooks, &PathBuf::from("/nonexistent"), &repo)
            .await
            .unwrap();
        assert_eq!(counts1.ebooks_found, 1);

        // Second scan should skip as duplicate
        let counts2 = LibraryScanner::scan(&ebooks, &PathBuf::from("/nonexistent"), &repo)
            .await
            .unwrap();
        assert_eq!(counts2.ebooks_found, 0);
        assert_eq!(counts2.duplicates_skipped, 1);
    }
}
