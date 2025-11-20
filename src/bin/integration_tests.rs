use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(name = "integration_tests")]
#[command(about = "Run integration tests for servicemaker")]
struct Args {
    /// Skip the test which runs the base image with mounting the zip file
    #[arg(long)]
    no_zip_test: bool,
}

#[derive(serde::Deserialize)]
struct TestConfig {
    base_image: String,
    entrypoint: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("=== Integration Tests ===\n");

    // Get the project root directory (assumed to be current directory)
    let project_root = std::env::current_dir()?;
    let testprojects_dir = project_root.join("testprojects");

    if !testprojects_dir.exists() {
        return Err(format!(
            "testprojects directory not found at: {}",
            testprojects_dir.display()
        )
        .into());
    }

    // List all subdirectories in testprojects
    let test_dirs = find_test_directories(&testprojects_dir)?;

    if test_dirs.is_empty() {
        return Err("No test directories found in testprojects/".into());
    }

    println!("Found {} test project(s):", test_dirs.len());
    for dir in &test_dirs {
        println!("  - {}", dir.display());
    }
    println!();

    let mut failed_projects: Vec<(String, String)> = Vec::new();

    // Process each test directory
    for test_dir in &test_dirs {
        let project_name = test_dir.file_name().unwrap().to_string_lossy().to_string();
        println!("=== Testing: {} ===", project_name);

        match test_project(&project_root, test_dir, args.no_zip_test) {
            Ok(_) => {
                println!("✓ Test passed for {}\n", project_name);
            }
            Err(e) => {
                let error_msg = e.to_string();
                eprintln!("✗ Test failed for {}: {}\n", project_name, error_msg);
                failed_projects.push((project_name, error_msg));
            }
        }
    }

    // Print summary
    println!("=== Test Summary ===");
    if failed_projects.is_empty() {
        println!("All tests passed!");
        Ok(())
    } else {
        println!("Failed projects:");
        for (name, error) in &failed_projects {
            println!("  - {}: {}", name, error);
        }
        Err(format!("{} test(s) failed", failed_projects.len()).into())
    }
}

fn find_test_directories(
    testprojects_dir: &Path,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut dirs = Vec::new();

    for entry in fs::read_dir(testprojects_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Check if it has a config.json file
            if path.join("config.json").exists() {
                dirs.push(path);
            }
        }
    }

    Ok(dirs)
}

fn test_project(
    project_root: &Path,
    test_dir: &Path,
    skip_zip_test: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_name = test_dir.file_name().unwrap().to_string_lossy().to_string();
    let config_path = test_dir.join("config.json");

    // Read config.json
    println!("Reading config from: {}", config_path.display());
    let config_content = fs::read_to_string(&config_path)?;
    let config: TestConfig = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config.json: {}", e))?;

    println!("  Base image: {}", config.base_image);
    println!("  Entrypoint: {}", config.entrypoint);

    // Determine paths
    let project_home = test_dir;
    let image_name = format!("arangodb/{}", project_name);
    let servicemaker_binary = project_root.join("target/release/servicemaker");

    if !servicemaker_binary.exists() {
        return Err(format!(
            "servicemaker binary not found at: {}. Please build with 'cargo build --release'",
            servicemaker_binary.display()
        )
        .into());
    }

    // Remove Docker image if it exists (to avoid conflicts)
    println!("\n--- Pre-test cleanup ---");
    remove_docker_image_if_exists(&image_name)?;

    // Run servicemaker
    println!("\nRunning servicemaker...");
    let servicemaker_status = Command::new(&servicemaker_binary)
        .args([
            "--name",
            &project_name,
            "--project-home",
            project_home.to_str().unwrap(),
            "--base-image",
            &config.base_image,
            "--image-name",
            &image_name,
            "--entrypoint",
            &config.entrypoint,
            "--port",
            "8080",
            "--make-tar-gz",
            // Note: push is false by default, so we don't need to specify it
        ])
        .current_dir(project_root)
        .status()?;

    if !servicemaker_status.success() {
        return Err(format!(
            "servicemaker failed with exit code: {:?}",
            servicemaker_status.code()
        )
        .into());
    }

    println!("✓ servicemaker completed successfully");

    // Find the temporary directory created by servicemaker
    let temp_dir_pattern = format!("servicemaker-{}-", project_name);
    let temp_dir = find_temp_directory(project_root, &temp_dir_pattern)?;
    println!("Found temporary directory: {}", temp_dir.display());

    // Test 1: Run Docker image directly
    println!("\n--- Test 1: Running Docker image ---");
    test_docker_image(&image_name)?;

    // Test 2: Run using tar.gz approach (skip if --no-zip-test is set)
    if skip_zip_test {
        println!("\n--- Test 2: Skipped (--no-zip-test flag set) ---");
    } else {
        println!("\n--- Test 2: Running with tar.gz file ---");
        let tar_file = temp_dir.join("project.tar.gz");
        if !tar_file.exists() {
            return Err(format!("project.tar.gz not found at: {}", tar_file.display()).into());
        }
        test_tar_gz_approach(&temp_dir, &tar_file, &config.base_image)?;
    }

    // Cleanup: Remove temporary directory and Docker image
    println!("\n--- Cleanup ---");
    cleanup_temp_directory(&temp_dir)?;
    cleanup_docker_image(&image_name)?;

    Ok(())
}

fn find_temp_directory(
    project_root: &Path,
    pattern: &str,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    for entry in fs::read_dir(project_root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir()
            && let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
            && dir_name.starts_with(pattern)
        {
            return Ok(path);
        }
    }

    Err(format!(
        "Could not find temporary directory matching pattern: {}",
        pattern
    )
    .into())
}

fn test_docker_image(image_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running: docker run --rm {}", image_name);

    let output = Command::new("docker")
        .args(["run", "--rm", image_name])
        .output()
        .map_err(|e| format!("Failed to run docker command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Docker run failed with exit code {:?}. Stderr: {}",
            output.status.code(),
            stderr
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Output:\n{}", stdout);

    if !stdout.contains("Hello World!") {
        return Err(format!(
            "Expected output to contain 'Hello World!', but got:\n{}",
            stdout
        )
        .into());
    }

    println!("✓ Docker image test passed");
    Ok(())
}

fn test_tar_gz_approach(
    temp_dir: &Path,
    tar_file: &Path,
    base_image: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get absolute path for the tar file
    let tar_file_abs = tar_file.canonicalize()?;

    println!(
        "Running: docker run --rm -v ./project.tar.gz:/project/project.tar.gz {}",
        base_image
    );
    println!("(from directory: {})", temp_dir.display());

    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-v",
            &format!("{}:/project/project.tar.gz", tar_file_abs.display()),
            base_image,
        ])
        .current_dir(temp_dir)
        .output()
        .map_err(|e| format!("Failed to run docker command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Docker run failed with exit code {:?}. Stderr: {}",
            output.status.code(),
            stderr
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Output:\n{}", stdout);

    // Check if the last line contains "Hello World!"
    let lines: Vec<&str> = stdout.lines().collect();
    if let Some(last_line) = lines.last() {
        if !last_line.contains("Hello World!") {
            return Err(format!(
                "Expected last line to contain 'Hello World!', but got:\n{}",
                stdout
            )
            .into());
        }
    } else {
        return Err("No output lines found".into());
    }

    println!("✓ tar.gz approach test passed");
    Ok(())
}

fn cleanup_temp_directory(temp_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("Removing temporary directory: {}", temp_dir.display());
    fs::remove_dir_all(temp_dir).map_err(|e| {
        format!(
            "Failed to remove temporary directory {}: {}",
            temp_dir.display(),
            e
        )
    })?;
    println!("✓ Temporary directory removed");
    Ok(())
}

fn remove_docker_image_if_exists(image_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Removing Docker image if it exists: {}", image_name);
    let output = Command::new("docker")
        .args(["rmi", image_name])
        .output()
        .map_err(|e| format!("Failed to run docker rmi command: {}", e))?;

    if output.status.success() {
        println!("✓ Docker image removed");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If image doesn't exist, that's fine - we're just cleaning up
        if stderr.contains("No such image") || stderr.contains("image not known") {
            println!("  (Docker image does not exist, skipping)");
        } else {
            // Other errors should be reported
            return Err(format!("Failed to remove Docker image {}: {}", image_name, stderr).into());
        }
    }

    Ok(())
}

fn cleanup_docker_image(image_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Removing Docker image: {}", image_name);
    let output = Command::new("docker")
        .args(["rmi", image_name])
        .output()
        .map_err(|e| format!("Failed to run docker rmi command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to remove Docker image {}: {}", image_name, stderr).into());
    }

    println!("✓ Docker image removed");
    Ok(())
}
