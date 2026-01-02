//! SSIM (Structural Similarity) visual regression tests
//!
//! Compares screenshots against baselines using SSIM algorithm.
//! SSIM values: 1.0 = identical, 0.0 = completely different
//! Threshold: 0.95+ is considered acceptable (minor rendering differences)

use image::DynamicImage;
use image_compare::{Algorithm, Similarity};
use std::fs;
use std::path::Path;

const BASELINE_DIR: &str = "screenshots/baseline";
const VERIFY_DIR: &str = "screenshots/verify";
const SSIM_THRESHOLD: f64 = 0.95;

/// Load image from path
fn load_image(path: &Path) -> Option<DynamicImage> {
    image::open(path).ok()
}

/// Compare two images using SSIM algorithm
fn compare_ssim(baseline: &DynamicImage, verify: &DynamicImage) -> f64 {
    let baseline_gray = baseline.to_luma8();
    let verify_gray = verify.to_luma8();

    // Resize if dimensions differ
    let (bw, bh) = baseline_gray.dimensions();
    let (vw, vh) = verify_gray.dimensions();

    if bw != vw || bh != vh {
        // Different dimensions, return low similarity
        return 0.0;
    }

    match image_compare::gray_similarity_structure(
        &Algorithm::MSSIMSimple,
        &baseline_gray,
        &verify_gray,
    ) {
        Ok(Similarity { score, .. }) => score,
        Err(_) => 0.0,
    }
}

/// Get all PNG files in a directory
fn get_png_files(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".png") {
                    files.push(name.to_string());
                }
            }
        }
    }
    files.sort();
    files
}

#[test]
fn test_ssim_visual_regression() {
    let baseline_path = Path::new(BASELINE_DIR);
    let verify_path = Path::new(VERIFY_DIR);

    // Skip if baseline doesn't exist
    if !baseline_path.exists() {
        println!("SSIM test skipped: no baseline directory at {BASELINE_DIR}");
        return;
    }

    let baseline_files = get_png_files(baseline_path);
    if baseline_files.is_empty() {
        println!("SSIM test skipped: no baseline screenshots");
        return;
    }

    // Skip if verify doesn't exist
    if !verify_path.exists() {
        println!("SSIM test skipped: no verify directory at {VERIFY_DIR}");
        return;
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut results = Vec::new();

    let verify_files = get_png_files(verify_path);
    if verify_files.is_empty() {
        println!("SSIM test skipped: no verify screenshots in {VERIFY_DIR}");
        return;
    }

    for filename in &baseline_files {
        let baseline_file = baseline_path.join(filename);
        let verify_file = verify_path.join(filename);

        if !verify_file.exists() {
            // Skip missing files silently - they may be new or renamed
            println!("SKIP: {filename} (not in verify)");
            continue;
        }

        let Some(baseline_img) = load_image(&baseline_file) else {
            println!("ERROR: Failed to load baseline {filename}");
            failed += 1;
            continue;
        };

        let Some(verify_img) = load_image(&verify_file) else {
            println!("ERROR: Failed to load verify {filename}");
            failed += 1;
            continue;
        };

        let ssim = compare_ssim(&baseline_img, &verify_img);
        results.push((filename.clone(), ssim));

        if ssim >= SSIM_THRESHOLD {
            println!("PASS: {filename} - SSIM: {ssim:.4}");
            passed += 1;
        } else {
            println!("FAIL: {filename} - SSIM: {ssim:.4} (threshold: {SSIM_THRESHOLD})");
            failed += 1;
        }
    }

    println!("\n=== SSIM Results ===");
    println!("Passed: {passed}, Failed: {failed}");
    println!("Threshold: {SSIM_THRESHOLD}");

    // Write results to JSON for tracking
    if !results.is_empty() {
        let json = serde_json::json!({
            "test": "ssim_visual_regression",
            "threshold": SSIM_THRESHOLD,
            "results": results.iter().map(|(name, ssim)| {
                serde_json::json!({
                    "file": name,
                    "ssim": ssim,
                    "passed": *ssim >= SSIM_THRESHOLD
                })
            }).collect::<Vec<_>>(),
            "summary": {
                "passed": passed,
                "failed": failed,
                "total": results.len()
            }
        });

        let _ = fs::create_dir_all("test_reports");
        let report_path = format!(
            "test_reports/ssim_{}.json",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );
        let _ = fs::write(
            &report_path,
            serde_json::to_string_pretty(&json).unwrap_or_default(),
        );
        println!("Report saved: {report_path}");
    }

    assert!(
        failed == 0,
        "SSIM visual regression failed: {failed} screenshots below threshold"
    );
}

#[test]
fn test_ssim_algorithm() {
    // Test SSIM algorithm with synthetic images
    use image::{GrayImage, Luma};

    // Create two identical images
    let img1 = GrayImage::from_fn(100, 100, |x, y| Luma([((x + y) % 256) as u8]));
    let img2 = img1.clone();

    let result = image_compare::gray_similarity_structure(&Algorithm::MSSIMSimple, &img1, &img2);

    match result {
        Ok(Similarity { score, .. }) => {
            assert!(
                (score - 1.0).abs() < 0.001,
                "Identical images should have SSIM ~1.0, got {score}"
            );
        }
        Err(e) => panic!("SSIM comparison failed: {e}"),
    }
}

#[test]
fn test_ssim_different_images() {
    // Test SSIM with different images
    use image::{GrayImage, Luma};

    let img1 = GrayImage::from_fn(100, 100, |_, _| Luma([0u8]));
    let img2 = GrayImage::from_fn(100, 100, |_, _| Luma([255u8]));

    let result = image_compare::gray_similarity_structure(&Algorithm::MSSIMSimple, &img1, &img2);

    match result {
        Ok(Similarity { score, .. }) => {
            assert!(
                score < 0.1,
                "Completely different images should have low SSIM, got {score}"
            );
        }
        Err(e) => panic!("SSIM comparison failed: {e}"),
    }
}
